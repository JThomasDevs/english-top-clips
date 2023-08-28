use std::fs;
use playwright::*;
use playwright::api::ElementHandle;


#[tokio::main]
async fn main() -> Result<(), Error> {
    let playwright = Playwright::initialize().await?;
    playwright.prepare()?; // Install browsers

    // Browser settings
    let webkit = playwright.webkit();
    let browser = webkit.launcher()
        .headless(true)
        .launch().await?;

    // Each context is a window, each page is a tab
    let context = browser.context_builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36")
        .build().await?;
    let page = context.new_page().await?;

    // Navigate to page
    page.goto_builder("https://www.twitch.tv/directory/category/just-chatting/clips?featured=false&range=7d")
        .goto().await?;
    println!("Clips page loaded");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Change language to English
    // open language menu
    page.query_selector("div[class='Layout-sc-1xcs6mc-0 fFENuB']").await?.unwrap().query_selector("button").await?.expect("Couldn't open language menu").click_builder().click().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    // click English
    page.query_selector("div[class='Layout-sc-1xcs6mc-0 gWkOOv']").await?.expect("Couldn't click on English").click_builder().click().await?;

    // Find all initial clips and then click a blank part of the page so the keyboard may be used
    let mut clips = page.query_selector_all("div[class='Layout-sc-1xcs6mc-0 iPAXTU']").await?;
    page.query_selector("div[data-a-target='root-scroller']").await?
        .expect("Can't find where to click :(")
        .click_builder()
        .click().await?;
    while clips.len() < 80 { // Gather 80 clip container elements in a vector
        page.keyboard
            .down("Space").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        clips = page.query_selector_all("div[class='Layout-sc-1xcs6mc-0 iPAXTU']").await?;
    }
    println!("Found {} clips", clips.len());
    // Get a link for each clip
    let (streamer_vec, url_vec) = clip_links(clips).await;
    // Navigate to third-party website to download clips
    // Close the webkit page
    page.close(Some(false)).await?;
    // Chromium settings
    let chromium = playwright.chromium();
    let browser = chromium.launcher()
        .headless(false)
        .downloads("C:\\Users\\jthom\\Desktop\\clips\\temp".as_ref())
        .launch().await?;
    let context = browser.context_builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36")
        .accept_downloads(true)
        .build().await?;

    // Clear old clips from clips folder if any exist
    let old_clips = fs::read_dir("C:\\Users\\jthom\\Desktop\\clips\\english clips").unwrap();
    for clip in old_clips {
        let _ = fs::remove_file(clip.unwrap().path());
    }
    // Clear any temp download files from the temp folder
    let temp_files = fs::read_dir("C:\\Users\\jthom\\Desktop\\clips\\temp").unwrap();
    for file in temp_files {
        let _ = fs::remove_file(file.unwrap().path());
    }
    println!("Old clips removed");
    // For each clip, navigate to clipsey.com and download it
    for (i, url) in url_vec.iter().enumerate() {
        // Open a new page
        let page = context.new_page().await?;
        // Navigate to clipsey.com
        page.goto_builder("https://clipsey.com/").goto().await?;
        let target = "https://www.twitch.tv".to_owned() + url.as_ref();
        println!("{}", &target);
        page.query_selector("input[class='clip-url-input']").await?
            .expect("Can't find URL input :(")
            .fill_builder(target.as_ref())
            .fill().await?;
        page.query_selector("button[class='get-download-link-button']").await?
            .expect("Can't find submit button :(")
            .click_builder()
            .click().await?;
        page.query_selector("a[class='download-clip-button button']").await?
            .expect("Can't find download button :(")
            .click_builder()
            .click().await?;

        // The downloaded clip has a random name, so we rename it to the clip ranking + the streamer's name
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let files = fs::read_dir("C:\\Users\\jthom\\Desktop\\clips\\temp").unwrap();
        for file in files {
            // Because can_rename is a result, it will be Err(false) until the file is able to be moved + renamed AKA when the download is finished
            loop {
                // In addition to checking if the rename is valid, it also performs the renaming action so once the file is able to be renamed and has been, the loop will break
                let can_rename = fs::rename(file.as_ref().unwrap().path(), format!("C:\\Users\\jthom\\Desktop\\clips\\english clips\\{}-{}.mp4", i+1, streamer_vec[i])).is_ok();
                if can_rename {
                    break;
                }
            }
        }
        println!("Downloaded clip {} of {}\n", i+1, url_vec.len());
        page.close(Some(false)).await?;
    }
    // End of program
    Ok(())
}

async fn clip_links(clip_vec: Vec<ElementHandle>) ->  (Vec<String>, Vec<String>) {
    // Vector to keep track of clip URLs
    let mut url_vec = Vec::new();
    // Vector to keep track of clip streamers
    let mut streamer_vec = Vec::new();
    // For each clip container, find the link to the clip, place it into the URL vector so that we can navigate to it later
    // Also, extract the streamer's name from each clip URL
    for (i, clip) in clip_vec.iter().enumerate() {
        let url = clip.query_selector("a[data-a-target='preview-card-image-link']").await
            .unwrap()
            .expect("Can't find clip link :(")
            .get_attribute("href").await
            .expect("Can't find clip link :(");
        println!("Creator of clip {}: {}", i+1, url.as_ref()
            .unwrap()
            .split('/')
            .nth(1)
            .unwrap());
        streamer_vec.push(url.as_ref()
            .unwrap().split('/')
            .nth(1)
            .unwrap()
            .to_string());
        url_vec.push(url.unwrap());
    }
    /*
     * Instead of navigating to each clip and downloading directly, we will use a third party website to download the clips
     * Therefore, we return the vectors so that we can use them later
     * The streamer_vec Vector is technically not needed because the streamer name could be extracted from the URL later
     * but this is easier for me so that's how it's going.
     */
    (streamer_vec, url_vec)
}