/// Open a new browser window with 2 tabs:
/// Tab 1: StadsAtlas with automated address entry workflow
/// Tab 2: Correlation result data
fn open_browser_windows(
    result: &&CorrelationResult,
    _window_idx: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = &result.address;

    // StadsAtlas URL with encoded address as parameter for auto-lookup
    let stadsatlas_url = format!(
        "https://stadsatlas.malmo.se/stadsatlas/?address={}",
        urlencoding::encode(address)
    );

    // Create correlation result data page
    let correlation_data = create_correlation_result_page(result);
    let correlation_data_url = format!(
        "data:text/html;charset=utf-8,{}",
        urlencoding::encode(&correlation_data)
    );

    // Try to open windows using different methods depending on OS
    #[cfg(target_os = "windows")]
    {
        // Windows: Open new browser window with both URLs
        std::process::Command::new("cmd")
            .args(&[
                "/C",
                &format!(
                    "start chrome \"{}\" && timeout /t 2 && start chrome \"{}\"",
                    stadsatlas_url, correlation_data_url
                ),
            ])
            .output()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: Open new Safari window with StadsAtlas, then correlation data in new tab
        let script = format!(
            r#"open '{}' & sleep 1 && open '{}' "#,
            stadsatlas_url, correlation_data_url
        );
        std::process::Command::new("bash")
            .args(&["-c", &script])
            .output()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: Open browser directly with StadsAtlas URL first, then correlation data
        let browser = get_browser_executable();

        // Open first tab with StadsAtlas
        std::process::Command::new(&browser)
            .arg(&stadsatlas_url)
            .spawn()
            .ok();

        // Small delay before opening second tab
        thread::sleep(Duration::from_millis(1000));

        // Open second tab with correlation data
        std::process::Command::new(&browser)
            .arg(&correlation_data_url)
            .spawn()
            .ok();
    }

    Ok(())
}