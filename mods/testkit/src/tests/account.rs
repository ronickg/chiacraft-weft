//! The account behind the player's key (chiacraft plan, phase 1): Profile leads to the Account
//! screen, which shows the 24 words on request, marks them written down and restores from words.
//! A test run keeps its account in `test-account/`, never the player's.

use std::time::Duration;

use weft_sdk::*;

use crate::runner::{R, T};

/// A valid 24-word phrase (BIP-39's all-zero entropy), for restoring.
const WORDS: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn shown(t: &T, text: &str) -> bool {
    t.c().ui_rect(&format!("text:{text}")).is_some()
}

async fn wait_text(t: &T, text: &str) -> R {
    let what = format!("\"{text}\" on screen");
    t.until(&what, Duration::from_secs(3), |t| shown(t, text))
        .await
}

pub async fn account(t: T) -> R {
    t.screen("core:title").await?;
    t.check(t.c().screen_open(&screens::PROFILE), "cannot open the profile");
    t.screen("core:profile").await?;
    t.frames(2).await?;
    t.shot("profile").await?;
    t.laid_out(&["text:from your account's 24 words", "text:Copy Key", "text:Account and Words"]);
    let needs_backup = shown(&t, "aren't written down");

    t.click_until_screen("text:Account and Words", "core:account")
        .await?;
    t.frames(2).await?;
    t.shot("account").await?;
    t.laid_out(&[
        "text:Account ",
        "text:Identity ",
        "text:Kept in a file",
        "text:Show My 24 Words",
        "text:Restore from words",
        "id:restore_words",
    ]);
    t.check_eq(shown(&t, "Write your 24 words down"), needs_backup, "the account screen agrees with the profile about the backup");

    // The words appear only on request, in 24 numbered places.
    t.check(!shown(&t, "Anyone with these words"), "the words are hidden until asked for");
    t.check(t.click_ui("text:Show My 24 Words", 0), "the Show My 24 Words button");
    wait_text(&t, "Anyone with these words").await?;
    t.frames(2).await?;
    t.shot("words").await?;
    t.laid_out(&["text:1. ", "text:12. ", "text:24. ", "text:Hide Words"]);
    t.check(!shown(&t, "25. "), "exactly 24 words");

    if needs_backup {
        t.check(t.click_ui("text:I Wrote Them Down", 0), "the I Wrote Them Down button");
        wait_text(&t, "marked as written down").await?;
        t.check(!shown(&t, "Anyone with these words"), "the words hide again once written down");
        t.check(!shown(&t, "Write your 24 words down"), "no backup reminder once written down");
        t.shot("backed_up").await?;
    } else {
        t.note("this run's account was already written down: the reminder is not checked");
        t.check(t.click_ui("text:Hide Words", 0), "the Hide Words button");
        t.until("the words hidden", Duration::from_secs(3), |t| !shown(t, "Anyone with these words"))
            .await?;
    }

    // Words that are not a phrase are refused; a phrase becomes the account from the next start.
    t.check(t.click_ui("id:restore_words", 0), "the restore field");
    t.frames(2).await?;
    t.c().input_type("these are not the words");
    t.frames(2).await?;
    t.check(t.click_ui("id:restore", 0), "the Restore button");
    t.frames(6).await?;
    t.shot("restore_refused").await?;
    t.check(!shown(&t, "Restored account"), "nonsense words restore nothing");

    // A fresh screen has an empty field.
    t.c().screen_close();
    t.screen("core:profile").await?;
    t.click_until_screen("text:Account and Words", "core:account")
        .await?;
    t.frames(2).await?;
    t.check(t.click_ui("id:restore_words", 0), "the restore field again");
    t.frames(2).await?;
    t.c().input_type(WORDS);
    t.frames(2).await?;
    t.check(t.click_ui("id:restore", 0), "the Restore button again");
    wait_text(&t, "Restored account").await?;
    t.shot("restored").await?;

    t.c().screen_close();
    t.screen("core:profile").await?;
    t.c().screen_close();
    t.screen("core:title").await?;
    Ok(())
}
