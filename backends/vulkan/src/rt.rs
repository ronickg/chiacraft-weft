//! Ray queries: bottom-level acceleration structures over triangles (built once, then compacted)
//! and top-level ones over their instances (built each frame), only where the device has
//! `VK_KHR_acceleration_structure` and `VK_KHR_ray_query` (`Backend::ray_query`).

use super::*;

/// Compacted sizes asked per frame slot; BLASes built past this many in a frame stay as built.
const MAX_COMPACT: u32 = 1024;
/// Bytes of one `VkAccelerationStructureInstanceKHR`.
const INSTANCE_BYTES: u64 = 64;

pub(crate) struct AccelMem {
    pub raw: vk::AccelerationStructureKHR,
    buffer: vk::Buffer,
    alloc: Allocation,
    pub size: u64,
    pub address: u64,
}

pub(crate) struct TlasSlot {
    mem: AccelMem,
    max: u32,
    scratch: u64,
    instances: (vk::Buffer, Allocation, u64),
    /// This frame's, written into `instances` when its build is recorded (a BLAS built earlier in
    /// the same frame is only then known to be built).
    pending: Vec<TlasInstance>,
    count: u32,
}

pub(crate) enum Accel {
    Blas {
        mem: AccelMem,
        geometries: Vec<BlasGeometry>,
        scratch: u64,
        built: bool,
        serial: u64,
    },
    Tlas {
        slots: Vec<TlasSlot>,
    },
}

pub(crate) struct Rt {
    pub accel_fn: khr::acceleration_structure::Device,
    scratch_align: u64,
    /// Per frame slot: the scratch builds use (buffer, memory, size, address).
    scratch: Vec<Option<(vk::Buffer, Allocation, u64, u64)>>,
    queries: vk::QueryPool,
    /// Per frame slot: the BLASes whose compacted size it asked (index, serial), in query order.
    asked: Vec<Vec<(u32, u64)>>,
    /// Smaller copies made for built BLASes, to copy into at the next frame's start.
    compactions: Vec<(u32, u64, AccelMem)>,
    serial: u64,
    pub accels: Slab<Accel>,
    pub memory: u64,
    /// Since the last log line: BLASes built, their triangles, bytes saved by compaction.
    built: (u32, u64, u64),
    logged: Option<std::time::Instant>,
}

/// Whether to use ray queries on a device that has them: not on the Mac (MoltenVK has none) or
/// Android (too slow to be worth it), and `WEFT_NO_RAY_QUERY=1` turns them off anywhere.
pub(crate) fn wanted() -> bool {
    !cfg!(any(target_os = "macos", target_os = "ios", target_os = "android")) && std::env::var_os("WEFT_NO_RAY_QUERY").is_none()
}

fn geometry_desc(g: &BlasGeometry, vertices: u64, indices: u64) -> vk::AccelerationStructureGeometryKHR<'static> {
    let triangles = vk::AccelerationStructureGeometryTrianglesDataKHR::default()
        .vertex_format(vk::Format::R32G32B32_SFLOAT)
        .vertex_data(vk::DeviceOrHostAddressConstKHR { device_address: vertices })
        .vertex_stride(g.vertex_stride as u64)
        .max_vertex(g.vertex_count.saturating_sub(1))
        .index_type(vk::IndexType::UINT32)
        .index_data(vk::DeviceOrHostAddressConstKHR { device_address: indices });
    vk::AccelerationStructureGeometryKHR::default()
        .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
        .geometry(vk::AccelerationStructureGeometryDataKHR { triangles })
        .flags(if g.opaque { vk::GeometryFlagsKHR::OPAQUE } else { vk::GeometryFlagsKHR::NO_DUPLICATE_ANY_HIT_INVOCATION })
}

fn instances_desc(address: u64) -> vk::AccelerationStructureGeometryKHR<'static> {
    let instances = vk::AccelerationStructureGeometryInstancesDataKHR::default()
        .array_of_pointers(false)
        .data(vk::DeviceOrHostAddressConstKHR { device_address: address });
    vk::AccelerationStructureGeometryKHR::default()
        .geometry_type(vk::GeometryTypeKHR::INSTANCES)
        .geometry(vk::AccelerationStructureGeometryDataKHR { instances })
}

impl Rt {
    pub unsafe fn new(instance: &ash::Instance, device: &ash::Device, pdev: vk::PhysicalDevice) -> Result<Rt, Error> {
        unsafe {
            let mut props = vk::PhysicalDeviceAccelerationStructurePropertiesKHR::default();
            let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut props);
            instance.get_physical_device_properties2(pdev, &mut props2);
            let queries = device
                .create_query_pool(
                    &vk::QueryPoolCreateInfo::default()
                        .query_type(vk::QueryType::ACCELERATION_STRUCTURE_COMPACTED_SIZE_KHR)
                        .query_count(MAX_COMPACT * FRAMES as u32),
                    None,
                )
                .map_err(err("compaction query pool"))?;
            Ok(Rt {
                accel_fn: khr::acceleration_structure::Device::new(instance, device),
                scratch_align: (props.min_acceleration_structure_scratch_offset_alignment as u64).max(1),
                scratch: (0..FRAMES).map(|_| None).collect(),
                queries,
                asked: vec![Vec::new(); FRAMES],
                compactions: Vec::new(),
                serial: 0,
                accels: Slab::new(),
                memory: 0,
                built: (0, 0, 0),
                logged: None,
            })
        }
    }
}

impl VulkanBackend {
    fn rt(&self) -> &Rt {
        self.rt.as_ref().expect("no ray queries on this device")
    }

    fn rt_mut(&mut self) -> &mut Rt {
        self.rt.as_mut().expect("no ray queries on this device")
    }

    pub(crate) fn buffer_address(&self, buffer: vk::Buffer) -> u64 {
        unsafe {
            self.device
                .get_buffer_device_address(&vk::BufferDeviceAddressInfo::default().buffer(buffer))
        }
    }

    fn accel_mem(&mut self, size: u64, ty: vk::AccelerationStructureTypeKHR, label: &str) -> AccelMem {
        let (buffer, alloc) = raw_buffer(
            &self.device,
            &mut self.allocator,
            self.spill,
            size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::GpuOnly,
            label,
        );
        let rt = self.rt.as_ref().unwrap();
        unsafe {
            let raw = rt
                .accel_fn
                .create_acceleration_structure(
                    &vk::AccelerationStructureCreateInfoKHR::default()
                        .buffer(buffer)
                        .size(size)
                        .ty(ty),
                    None,
                )
                .expect("vkCreateAccelerationStructureKHR");
            let address = rt
                .accel_fn
                .get_acceleration_structure_device_address(&vk::AccelerationStructureDeviceAddressInfoKHR::default().acceleration_structure(raw));
            self.rt.as_mut().unwrap().memory += size;
            AccelMem { raw, buffer, alloc, size, address }
        }
    }

    /// Frees `mem` once the frames that may use it are done.
    fn defer_accel(&mut self, mem: AccelMem) {
        self.rt_mut().memory -= mem.size;
        self.defer(Deferred::Accel(mem.raw, mem.buffer, mem.alloc));
    }

    pub(crate) unsafe fn destroy_accel_now(&mut self, raw: vk::AccelerationStructureKHR, buffer: vk::Buffer, alloc: Allocation) {
        unsafe {
            if let Some(rt) = &self.rt {
                rt.accel_fn.destroy_acceleration_structure(raw, None);
            }
            self.device.destroy_buffer(buffer, None);
            self.allocator.free(alloc).ok();
        }
    }

    pub(crate) fn rt_create_blas(&mut self, geometries: &[BlasGeometry], label: &str) -> AccelId {
        let descs: Vec<_> = geometries
            .iter()
            .map(|g| geometry_desc(g, 0, 0))
            .collect();
        let counts: Vec<u32> = geometries.iter().map(|g| g.triangles).collect();
        let info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE | vk::BuildAccelerationStructureFlagsKHR::ALLOW_COMPACTION)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .geometries(&descs);
        let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();
        unsafe {
            self.rt().accel_fn.get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &info,
                &counts,
                &mut sizes,
            )
        };
        let mem = self.accel_mem(sizes.acceleration_structure_size, vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL, label);
        let rt = self.rt_mut();
        rt.serial += 1;
        let serial = rt.serial;
        AccelId(rt.accels.insert(Accel::Blas {
            mem,
            geometries: geometries.to_vec(),
            scratch: sizes.build_scratch_size,
            built: false,
            serial,
        }))
    }

    fn tlas_slot(&mut self, max: u32, label: &str) -> TlasSlot {
        let max = max.max(16);
        let desc = [instances_desc(0)];
        let info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
            .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .geometries(&desc);
        let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();
        unsafe {
            self.rt().accel_fn.get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &info,
                &[max],
                &mut sizes,
            )
        };
        let mem = self.accel_mem(sizes.acceleration_structure_size, vk::AccelerationStructureTypeKHR::TOP_LEVEL, label);
        let (buffer, alloc) = raw_buffer(
            &self.device,
            &mut self.allocator,
            self.spill,
            max as u64 * INSTANCE_BYTES,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::CpuToGpu,
            label,
        );
        let address = self.buffer_address(buffer);
        TlasSlot { mem, max, scratch: sizes.build_scratch_size, instances: (buffer, alloc, address), pending: Vec::new(), count: 0 }
    }

    pub(crate) fn rt_create_tlas(&mut self, max: u32, label: &str) -> AccelId {
        let slots = (0..FRAMES).map(|_| self.tlas_slot(max, label)).collect();
        AccelId(self.rt_mut().accels.insert(Accel::Tlas { slots }))
    }

    pub(crate) fn rt_set_instances(&mut self, tlas: AccelId, instances: &[TlasInstance]) {
        let slot = self.slot;
        if let Accel::Tlas { slots } = self.rt_mut().accels.get_mut(tlas.0) {
            slots[slot].pending.clear();
            slots[slot].pending.extend_from_slice(instances);
        }
    }

    pub(crate) fn rt_destroy(&mut self, accel: AccelId) {
        let Some(a) = self.rt_mut().accels.remove(accel.0) else { return };
        match a {
            Accel::Blas { mem, .. } => self.defer_accel(mem),
            Accel::Tlas { slots } => {
                for s in slots {
                    self.defer_accel(s.mem);
                    self.defer(Deferred::Buffer(s.instances.0, s.instances.1));
                }
            }
        }
    }

    /// The slot's scratch, at least `size` bytes: its address.
    fn scratch(&mut self, size: u64) -> u64 {
        let slot = self.slot;
        if let Some(s) = &self.rt().scratch[slot]
            && s.2 >= size
        {
            return s.3;
        }
        if let Some((b, a, ..)) = self.rt_mut().scratch[slot].take() {
            // Builds recorded earlier in this frame may still use it.
            self.defer(Deferred::Buffer(b, a));
        }
        let size = size.next_power_of_two().max(1 << 20);
        let (buffer, alloc) = raw_buffer(
            &self.device,
            &mut self.allocator,
            self.spill,
            size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::GpuOnly,
            "acceleration structure scratch",
        );
        let address = self.buffer_address(buffer);
        self.rt_mut().scratch[slot] = Some((buffer, alloc, size, address));
        address
    }

    /// Run once the slot's last frame is done (`ensure_slot_ready`): the compacted sizes it asked
    /// come back, and each BLAS that shrinks gets its smaller copy, filled at the next frame's start.
    pub(crate) fn rt_slot_ready(&mut self) {
        let slot = self.slot;
        let Some(rt) = self.rt.as_mut() else { return };
        let asked = std::mem::take(&mut rt.asked[slot]);
        if asked.is_empty() {
            return;
        }
        let mut sizes = vec![0u64; asked.len()];
        let ok = unsafe {
            self.device.get_query_pool_results(
                rt.queries,
                slot as u32 * MAX_COMPACT,
                &mut sizes,
                vk::QueryResultFlags::TYPE_64 | vk::QueryResultFlags::WAIT,
            )
        }
        .is_ok();
        if !ok {
            return;
        }
        for ((index, serial), compact) in asked.into_iter().zip(sizes) {
            let rt = self.rt.as_ref().unwrap();
            let Some(Some(Accel::Blas { mem, serial: s, .. })) = rt.accels.items.get(index as usize) else { continue };
            // Only worth a copy if it saves at least a tenth.
            if *s != serial || compact == 0 || compact * 10 > mem.size * 9 {
                continue;
            }
            let new = self.accel_mem(compact, vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL, "compacted BLAS");
            self.rt_mut().compactions.push((index, serial, new));
        }
    }

    /// At the frame's start, after the uploads: the compactions, and the slot's queries reset.
    pub(crate) unsafe fn rt_record_start(&mut self, cmd: vk::CommandBuffer) {
        let slot = self.slot;
        let Some(rt) = self.rt.as_mut() else { return };
        unsafe {
            self.device
                .cmd_reset_query_pool(cmd, rt.queries, slot as u32 * MAX_COMPACT, MAX_COMPACT);
        }
        let compactions = std::mem::take(&mut rt.compactions);
        if compactions.is_empty() {
            return;
        }
        let mut old_mems = Vec::new();
        let mut dead = Vec::new();
        for (index, serial, new) in compactions {
            let rt = self.rt.as_mut().unwrap();
            match rt.accels.items.get_mut(index as usize) {
                Some(Some(Accel::Blas { mem, serial: s, .. })) if *s == serial => {
                    unsafe {
                        rt.accel_fn.cmd_copy_acceleration_structure(
                            cmd,
                            &vk::CopyAccelerationStructureInfoKHR::default()
                                .src(mem.raw)
                                .dst(new.raw)
                                .mode(vk::CopyAccelerationStructureModeKHR::COMPACT),
                        );
                    }
                    rt.built.2 += mem.size - new.size;
                    old_mems.push(std::mem::replace(mem, new));
                }
                _ => dead.push(new),
            }
        }
        for m in old_mems.into_iter().chain(dead) {
            self.defer_accel(m);
        }
        unsafe { self.memory_barrier(cmd) };
    }

    /// Records the builds of these BLASes, as many at once as the scratch allows.
    pub(crate) unsafe fn rt_build_blases(&mut self, cmd: vk::CommandBuffer, ids: &[u32]) {
        let align = self.rt().scratch_align;
        // Total scratch for the batch, capped: past it the batch is split, a barrier between.
        const CAP: u64 = 256 << 20;
        let mut todo: Vec<(u32, Vec<BlasGeometry>, u64)> = Vec::new();
        for &id in ids {
            if let Some(Some(Accel::Blas { geometries, scratch, built: false, .. })) = self.rt().accels.items.get(id as usize) {
                todo.push((id, geometries.clone(), *scratch));
            }
        }
        let mut start = 0;
        while start < todo.len() {
            let mut total = 0u64;
            let mut end = start;
            while end < todo.len() && (end == start || total + todo[end].2 + align <= CAP) {
                total += todo[end].2.next_multiple_of(align) + align;
                end += 1;
            }
            let base = self.scratch(total);
            let mut geoms: Vec<Vec<vk::AccelerationStructureGeometryKHR>> = Vec::new();
            let mut ranges: Vec<Vec<vk::AccelerationStructureBuildRangeInfoKHR>> = Vec::new();
            let mut dsts = Vec::new();
            let mut scratch_at = Vec::new();
            let mut offset = base.next_multiple_of(align) - base;
            for (id, gs, size) in &todo[start..end] {
                let mut g = Vec::new();
                let mut r = Vec::new();
                for geo in gs {
                    let v = self.buffer_address(self.buffers.get(geo.vertices.0).raw) + geo.vertex_offset;
                    let i = self.buffer_address(self.buffers.get(geo.indices.0).raw) + geo.index_offset;
                    g.push(geometry_desc(geo, v, i));
                    r.push(vk::AccelerationStructureBuildRangeInfoKHR::default().primitive_count(geo.triangles));
                    self.rt_mut().built.1 += geo.triangles as u64;
                }
                geoms.push(g);
                ranges.push(r);
                let Accel::Blas { mem, .. } = self.rt().accels.get(*id) else { unreachable!() };
                dsts.push(mem.raw);
                scratch_at.push(base + offset);
                offset += size.next_multiple_of(align) + align;
            }
            let infos: Vec<_> = geoms
                .iter()
                .zip(&dsts)
                .zip(&scratch_at)
                .map(|((g, &dst), &s)| {
                    vk::AccelerationStructureBuildGeometryInfoKHR::default()
                        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                        .flags(
                            vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE
                                | vk::BuildAccelerationStructureFlagsKHR::ALLOW_COMPACTION,
                        )
                        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                        .dst_acceleration_structure(dst)
                        .geometries(g)
                        .scratch_data(vk::DeviceOrHostAddressKHR { device_address: s })
                })
                .collect();
            let range_refs: Vec<&[vk::AccelerationStructureBuildRangeInfoKHR]> = ranges.iter().map(|r| &r[..]).collect();
            unsafe {
                self.rt()
                    .accel_fn
                    .cmd_build_acceleration_structures(cmd, &infos, &range_refs);
                self.memory_barrier(cmd);
            }
            // Ask for their compacted sizes, read once this frame is done (`rt_slot_ready`).
            let slot = self.slot;
            let rt = self.rt.as_mut().unwrap();
            let room = (MAX_COMPACT as usize).saturating_sub(rt.asked[slot].len());
            let mut asked = Vec::new();
            for (id, ..) in &todo[start..end] {
                if let Accel::Blas { built, serial, .. } = rt.accels.get_mut(*id) {
                    *built = true;
                    if asked.len() < room {
                        asked.push((*id, *serial));
                    }
                }
            }
            rt.built.0 += (end - start) as u32;
            if !asked.is_empty() {
                let raws: Vec<_> = asked
                    .iter()
                    .map(|(id, _)| match rt.accels.get(*id) {
                        Accel::Blas { mem, .. } => mem.raw,
                        Accel::Tlas { .. } => unreachable!(),
                    })
                    .collect();
                unsafe {
                    rt.accel_fn.cmd_write_acceleration_structures_properties(
                        cmd,
                        &raws,
                        vk::QueryType::ACCELERATION_STRUCTURE_COMPACTED_SIZE_KHR,
                        rt.queries,
                        slot as u32 * MAX_COMPACT + rt.asked[slot].len() as u32,
                    );
                }
                rt.asked[slot].extend(asked);
            }
            start = end;
        }
        self.rt_log();
    }

    /// Records the build of `tlas` from this frame's instances.
    pub(crate) unsafe fn rt_build_tlas(&mut self, cmd: vk::CommandBuffer, tlas: u32) {
        let slot = self.slot;
        let Some(Some(Accel::Tlas { slots })) = self.rt_mut().accels.items.get_mut(tlas as usize) else { return };
        let pending = std::mem::take(&mut slots[slot].pending);
        let max = slots[slot].max;
        if pending.len() as u32 > max {
            let grown = self.tlas_slot((pending.len() as u32).next_power_of_two(), "TLAS");
            let Accel::Tlas { slots } = self.rt_mut().accels.get_mut(tlas) else { unreachable!() };
            let old = std::mem::replace(&mut slots[slot], grown);
            self.defer_accel(old.mem);
            self.defer(Deferred::Buffer(old.instances.0, old.instances.1));
        }
        // The instances, with the BLASes' addresses as they are now (after compaction): one not
        // built yet is left out.
        let mut bytes = Vec::with_capacity(pending.len() * INSTANCE_BYTES as usize);
        let mut count = 0u32;
        {
            let rt = self.rt();
            for inst in &pending {
                let Some(Some(Accel::Blas { mem, built: true, .. })) = rt.accels.items.get(inst.blas.0 as usize) else { continue };
                for row in &inst.transform {
                    for v in row {
                        bytes.extend_from_slice(&v.to_le_bytes());
                    }
                }
                bytes.extend_from_slice(&((inst.custom & 0xff_ffff) | (inst.mask as u32) << 24).to_le_bytes());
                // Both sides of every triangle: a shadow ray leaves the far side of a leaf too.
                let flags = vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw();
                bytes.extend_from_slice(&(flags << 24).to_le_bytes());
                bytes.extend_from_slice(&mem.address.to_le_bytes());
                count += 1;
            }
        }
        let Accel::Tlas { slots } = self.rt_mut().accels.get_mut(tlas) else { unreachable!() };
        let s = &mut slots[slot];
        s.instances.1.mapped_slice_mut().unwrap()[..bytes.len()].copy_from_slice(&bytes);
        s.count = count;
        let (dst, scratch_size, address) = (s.mem.raw, s.scratch, s.instances.2);
        let scratch = self.scratch(scratch_size + self.rt().scratch_align);
        let scratch = scratch.next_multiple_of(self.rt().scratch_align);
        let desc = [instances_desc(address)];
        let info = vk::AccelerationStructureBuildGeometryInfoKHR::default()
            .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .dst_acceleration_structure(dst)
            .geometries(&desc)
            .scratch_data(vk::DeviceOrHostAddressKHR { device_address: scratch });
        let range = [vk::AccelerationStructureBuildRangeInfoKHR::default().primitive_count(count)];
        unsafe {
            self.rt()
                .accel_fn
                .cmd_build_acceleration_structures(cmd, &[info], &[&range]);
            self.memory_barrier(cmd);
        }
    }

    /// The TLAS `binding` should see this frame.
    pub(crate) fn rt_tlas_raw(&self, tlas: u32) -> Option<vk::AccelerationStructureKHR> {
        match self.rt.as_ref()?.accels.items.get(tlas as usize)? {
            Some(Accel::Tlas { slots }) => Some(slots[self.slot].mem.raw),
            _ => None,
        }
    }

    /// Once every few seconds while BLASes are built: how many, their triangles and memory.
    fn rt_log(&mut self) {
        let rt = self.rt.as_mut().unwrap();
        let now = std::time::Instant::now();
        let since = *rt.logged.get_or_insert(now);
        if now - since < std::time::Duration::from_secs(5) {
            return;
        }
        log::info!(
            target: "rt",
            "BLAS: {} built ({} triangles) in {:.1} s, compaction saved {:.1} MB, acceleration structures {:.1} MB",
            rt.built.0,
            rt.built.1,
            (now - since).as_secs_f32(),
            rt.built.2 as f64 / 1048576.0,
            rt.memory as f64 / 1048576.0
        );
        rt.built = (0, 0, 0);
        rt.logged = Some(now);
    }

    pub(crate) unsafe fn rt_drop(&mut self) {
        let Some(mut rt) = self.rt.take() else { return };
        unsafe {
            for (b, a, ..) in rt.scratch.drain(..).flatten() {
                self.device.destroy_buffer(b, None);
                self.allocator.free(a).ok();
            }
            let mut mems = Vec::new();
            for i in 0..rt.accels.items.len() as u32 {
                match rt.accels.remove(i) {
                    Some(Accel::Blas { mem, .. }) => mems.push(mem),
                    Some(Accel::Tlas { slots }) => {
                        for s in slots {
                            mems.push(s.mem);
                            self.device.destroy_buffer(s.instances.0, None);
                            self.allocator.free(s.instances.1).ok();
                        }
                    }
                    None => {}
                }
            }
            mems.extend(rt.compactions.drain(..).map(|c| c.2));
            for m in mems {
                rt.accel_fn.destroy_acceleration_structure(m.raw, None);
                self.device.destroy_buffer(m.buffer, None);
                self.allocator.free(m.alloc).ok();
            }
            self.device.destroy_query_pool(rt.queries, None);
        }
    }
}
