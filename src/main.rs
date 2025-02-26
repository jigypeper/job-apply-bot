use clap::Parser;
use dotenv::dotenv;
use std::env;
use std::time::Duration;
use tempfile::TempDir;
use thirtyfour::prelude::*;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The LinkedIn job search URL
    job_url: String,
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    dotenv().ok(); // Load environment variables from .env file

    let linkedin_id = env::var("LINKEDIN_ID").expect("Missing LINKEDIN_ID");
    let linkedin_key = env::var("LINKEDIN_KEY").expect("Missing LINKEDIN_KEY");

    let args = Args::parse();
    let job_url = args.job_url;

    // Set up Chromium with headless mode
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    // Set a unique user data directory to avoid conflicts
    // Create a unique temporary directory
    let tmp_dir = TempDir::new()?;
    let user_data_dir = tmp_dir.path().to_str().unwrap();

    caps.add_arg(format!("--user-data-dir={}", user_data_dir).as_str())?;
    println!("Starting job application bot...");
    println!("Job search URL: {}", job_url);

    let driver = WebDriver::new("http://localhost:9515", caps).await?;

    println!("Created Driver");
    // Open LinkedIn Jobs page
    driver.get(&job_url).await?;
    sleep(Duration::from_secs(5)).await;

    // Click 'Sign in'
    let login_button = driver.find(By::LinkText("Sign in")).await?;
    login_button.click().await?;
    sleep(Duration::from_secs(2)).await;

    // Enter credentials
    driver
        .find(By::Css("#username"))
        .await?
        .send_keys(&linkedin_id)
        .await?;
    driver
        .find(By::Css("#password"))
        .await?
        .send_keys(&linkedin_key)
        .await?;
    driver
        .find(By::Css("#password"))
        .await?
        .send_keys(Key::Enter)
        .await?;
    sleep(Duration::from_secs(5)).await; // Allow login to complete

    println!("Logged in successfully!");

    // Get job listings
    let job_listings = driver
        .query(By::Css(".job-card-list__title"))
        .all_from_selector()
        .await?;
    println!("Found {} job listings.", job_listings.len());

    for listing in job_listings {
        listing.click().await?;
        sleep(Duration::from_secs(2)).await;

        // Try to find the "Easy Apply" button
        if let Ok(apply_button) = driver.find(By::Css(".jobs-s-apply button")).await {
            apply_button.click().await?;
            sleep(Duration::from_secs(5)).await;

            // Check if it's a single-click apply or multi-step
            if let Ok(submit_button) = driver.find(By::Css("footer button")).await {
                let button_text = submit_button.text().await?;
                if button_text.contains("Next") {
                    // Multi-step application, skip it
                    if let Ok(close_button) =
                        driver.find(By::ClassName("artdeco-modal__dismiss")).await
                    {
                        close_button.click().await?;
                        sleep(Duration::from_secs(2)).await;

                        // Handle multiple discard buttons properly
                        if let Ok(discard_buttons) = driver
                            .query(By::ClassName("artdeco-modal__confirm-dialog-btn"))
                            .all_from_selector()
                            .await
                        {
                            if discard_buttons.len() > 1 {
                                if let Some(discard_button) = discard_buttons.get(1) {
                                    discard_button.click().await?;
                                }
                            } else if let Some(first_discard) = discard_buttons.first() {
                                first_discard.click().await?;
                            }
                        }
                    }
                    println!("Skipped multi-step application.");
                    continue;
                } else {
                    submit_button.click().await?;
                    println!("Applied to job!");
                }
            }

            // Close pop-up after applying
            if let Ok(close_button) = driver.find(By::ClassName("artdeco-modal__dismiss")).await {
                close_button.click().await?;
            }

            sleep(Duration::from_secs(2)).await;
        } else {
            println!("No 'Easy Apply' button found, skipped.");
            continue;
        }
    }

    driver.quit().await?;
    Ok(())
}
