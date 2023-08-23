use playwright::api::{ElementHandle, Page};
use playwright::Playwright;

#[tokio::main]
async fn main() -> Result<(), playwright::Error> {
    let playwright = Playwright::initialize().await?;
    playwright.prepare()?; // Install browsers

    let webkit = playwright.chromium();
    let browser = webkit
        .launcher()
        .headless(false)
        .downloads("C:\\Users\\jthom\\Desktop\\clips\\temp".as_ref())
        .launch().await?;
    let context = browser
        .context_builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36")
        .accept_downloads(true)
        .build().await?;
    let page = context
        .new_page().await?;

    // Navigate to the top English clips of the past 7 days
    page
        .goto_builder("https://streamscharts.com/clips?language=en")
        .goto().await?;

    // Find all clips on the page
    let clips_container = page.query_selector("div[class='grid-cols-12 gap-6  grid ']")
        .await?.unwrap();
    let mut clips = clips_container
        .query_selector_all("button")
        .await?;
    let mut seen_clips = clips_container
        .query_selector_all("button")
        .await?; // Used to remove seen clips from the clips vector

    // While we don't yet have 100 embeds
    let mut streamer_vec: Vec<String> = Vec::new();
    let mut embed_vec: Vec<String> = Vec::new();
    while (streamer_vec.len() < 100) && (embed_vec.len() < 100) {
        // If seen_clips and clips vector are different lengths, remove the clips that were already seen
        println!("{} clips", clips.len());
        println!("{} seen clips", seen_clips.len());
        // TODO: Seen clips are not being removed from the clips vector and I need to figure out why
        if seen_clips.len() != clips.len() {
            for (i, clip) in seen_clips.iter().enumerate() {
                if seen_clips[i] == clips[i] {
                    clips.remove(i);
                }
            }
        }
        // Re-assign seen_clips with the clips now visible on the page
        // Redundant only on the first iteration
        seen_clips = clips_container
            .query_selector_all("button")
            .await?;
        // Get the currently visible streamer names and embed links
        let (temp_streamer, temp_embed) = get_clip_embeds(&page, &clips).await;
        // Append the gathered info to the appropriate vectors
        streamer_vec.append(&mut temp_streamer.clone());
        embed_vec.append(&mut temp_embed.clone());
        // After gathering the info, we should be at the bottom of the page, so click the "Show More" button
       page.query_selector("div[class='relative flex justify-center mt-6']").await?.unwrap().query_selector("button").await?.unwrap().click_builder().click().await?;
        tokio::time::sleep(std::time::Duration::from_millis(3500)).await;

        // Re-assign clips vector with the newly visible clips
        // This includes clips which have already been seen so they will be removed from the clips vector at the start of the next iteration
        clips = clips_container
            .query_selector_all("button")
            .await?;
    }

    // End of program
    Ok(())
}

async fn get_clip_embeds(page: &Page ,clips: &Vec<ElementHandle>) -> (Vec<String>, Vec<String>) {
    let mut streamer_vec = Vec::new();
    let mut embed_vec = Vec::new();
    // Iterate through all clips
    for clip in clips {
        let streamer = clip.query_selector("div[class='text-sm font-bold truncate text-default']").await.unwrap().unwrap().text_content().await.unwrap().unwrap().split("\n").collect::<Vec<&str>>()[1].to_string(); // Streamer's name
        println!("{}", streamer);
        streamer_vec.push(streamer);
        // Open the clip
        let _ = clip.click_builder()
            .click().await;
        // Wait 2 seconds to prevent fuckery
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        // Clip player container
        let big_clip = page.query_selector("div[class='relative w-full video-popup-position max-h-70h md:max-w-70w md:h-full']").await.unwrap().unwrap();
        // Extract the embed link
        let embed = big_clip.query_selector("iframe[class='ratio_container-inner']").await.unwrap().unwrap().get_attribute("src").await.unwrap().unwrap();
        println!("{}", embed);
        embed_vec.push(embed);
        // Close the clip
        let _ = big_clip.query_selector("svg[title='close']").await.unwrap().unwrap().click_builder().click().await;
    }
    (streamer_vec, embed_vec)
}