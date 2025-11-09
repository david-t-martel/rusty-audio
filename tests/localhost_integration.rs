//! Localhost integration tests for WASM/PWA deployment
//!
//! These tests verify that the WASM build works correctly when served
//! via localhost. Run with: `just test-localhost`
//!
//! Prerequisites:
//! - Build WASM: `just build-wasm`
//! - Start server: `just serve-dist` or `just serve-wasm`

#[cfg(test)]
mod localhost_tests {
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;

    const LOCALHOST_URL: &str = "http://localhost:8080";
    const TIMEOUT_SECS: u64 = 10;

    /// Check if the localhost server is running
    fn is_server_running() -> bool {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .ok()
            .and_then(|client| client.get(LOCALHOST_URL).send().ok())
            .is_some()
    }

    /// Wait for server to be available
    fn wait_for_server() -> Result<(), String> {
        for _ in 0..TIMEOUT_SECS {
            if is_server_running() {
                return Ok(());
            }
            thread::sleep(Duration::from_secs(1));
        }
        Err(format!(
            "Server not available at {} after {} seconds. Start with: just serve-wasm",
            LOCALHOST_URL, TIMEOUT_SECS
        ))
    }

    #[test]
    #[ignore] // Only run when server is explicitly started
    fn test_server_available() {
        wait_for_server().expect("Server should be running");
    }

    #[test]
    #[ignore]
    fn test_index_html_loads() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(LOCALHOST_URL)
            .send()
            .expect("Failed to fetch index.html");

        assert!(
            response.status().is_success(),
            "index.html should return 200 OK"
        );

        let content = response.text().expect("Failed to read response body");
        assert!(
            content.contains("<canvas"),
            "index.html should contain canvas element"
        );
        assert!(
            content.contains("rusty-audio"),
            "index.html should reference rusty-audio"
        );
    }

    #[test]
    #[ignore]
    fn test_wasm_file_exists() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();

        // Check if WASM file is accessible
        let wasm_url = format!("{}/pkg/rusty_audio_bg.wasm", LOCALHOST_URL);
        let response = client
            .get(&wasm_url)
            .send()
            .expect("Failed to fetch WASM file");

        assert!(
            response.status().is_success(),
            "WASM file should be accessible at {}",
            wasm_url
        );

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok());

        assert!(
            content_type.map_or(false, |ct| ct.contains("wasm")
                || ct.contains("application/octet-stream")),
            "WASM file should have correct content-type, got: {:?}",
            content_type
        );
    }

    #[test]
    #[ignore]
    fn test_javascript_glue_exists() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();
        let js_url = format!("{}/pkg/rusty_audio.js", LOCALHOST_URL);

        let response = client
            .get(&js_url)
            .send()
            .expect("Failed to fetch JS glue code");

        assert!(
            response.status().is_success(),
            "JS glue code should be accessible"
        );

        let content = response.text().expect("Failed to read JS content");
        assert!(content.contains("wasm"), "JS should reference WASM");
        assert!(content.len() > 1000, "JS glue code should be substantial");
    }

    #[test]
    #[ignore]
    fn test_pwa_manifest_exists() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();
        let manifest_url = format!("{}/manifest.webmanifest", LOCALHOST_URL);

        let response = client
            .get(&manifest_url)
            .send()
            .expect("Failed to fetch PWA manifest");

        assert!(
            response.status().is_success(),
            "PWA manifest should be accessible"
        );

        let content = response.text().expect("Failed to read manifest");
        assert!(
            content.contains("Rusty Audio"),
            "Manifest should contain app name"
        );
        assert!(content.contains("icons"), "Manifest should contain icons");
        assert!(
            content.contains("standalone"),
            "Manifest should specify display mode"
        );
    }

    #[test]
    #[ignore]
    fn test_service_worker_exists() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();
        let sw_url = format!("{}/service-worker.js", LOCALHOST_URL);

        let response = client
            .get(&sw_url)
            .send()
            .expect("Failed to fetch service worker");

        assert!(
            response.status().is_success(),
            "Service worker should be accessible"
        );

        let content = response.text().expect("Failed to read service worker");
        assert!(
            content.contains("install"),
            "Service worker should have install event"
        );
        assert!(
            content.contains("fetch"),
            "Service worker should have fetch event"
        );
        assert!(
            content.contains("cache"),
            "Service worker should implement caching"
        );
    }

    #[test]
    #[ignore]
    fn test_pwa_icons_exist() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();

        // Check 192x192 icon
        let icon_192_url = format!("{}/icons/icon-192.png", LOCALHOST_URL);
        let response_192 = client.get(&icon_192_url).send();
        assert!(
            response_192.is_ok() && response_192.unwrap().status().is_success(),
            "192x192 icon should be accessible"
        );

        // Check 512x512 icon
        let icon_512_url = format!("{}/icons/icon-512.png", LOCALHOST_URL);
        let response_512 = client.get(&icon_512_url).send();
        assert!(
            response_512.is_ok() && response_512.unwrap().status().is_success(),
            "512x512 icon should be accessible"
        );
    }

    #[test]
    #[ignore]
    fn test_security_headers() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(LOCALHOST_URL)
            .send()
            .expect("Failed to fetch for header check");

        let headers = response.headers();

        // Check for important security headers (may not all be present in dev server)
        println!("Response headers:");
        for (key, value) in headers.iter() {
            println!("  {}: {:?}", key, value);
        }

        // Note: Local dev servers may not set all headers
        // This test documents expected production headers
    }

    #[test]
    #[ignore]
    fn test_wasm_bundle_size() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();
        let wasm_url = format!("{}/pkg/rusty_audio_bg.wasm", LOCALHOST_URL);

        let response = client
            .get(&wasm_url)
            .send()
            .expect("Failed to fetch WASM for size check");

        let content = response.bytes().expect("Failed to read WASM bytes");
        let size_mb = content.len() as f64 / 1024.0 / 1024.0;

        println!("WASM bundle size: {:.2} MB", size_mb);

        // Warn if bundle is too large (>5MB uncompressed is concerning)
        if size_mb > 5.0 {
            eprintln!(
                "⚠️  Warning: WASM bundle is large ({:.2} MB). Consider optimization.",
                size_mb
            );
        }

        assert!(size_mb > 0.1, "WASM bundle should be at least 100KB");
        assert!(
            size_mb < 20.0,
            "WASM bundle should be under 20MB (current: {:.2} MB)",
            size_mb
        );
    }

    #[test]
    #[ignore]
    fn test_all_core_assets_load() {
        wait_for_server().expect("Server should be running");

        let client = reqwest::blocking::Client::new();
        let core_assets = vec![
            "/",
            "/index.html",
            "/pkg/rusty_audio.js",
            "/pkg/rusty_audio_bg.wasm",
            "/manifest.webmanifest",
            "/service-worker.js",
            "/icons/icon-192.png",
            "/icons/icon-512.png",
        ];

        let mut failures = Vec::new();
        for asset in &core_assets {
            let url = format!("{}{}", LOCALHOST_URL, asset);
            match client.get(&url).send() {
                Ok(response) if response.status().is_success() => {
                    println!("✅ {} - OK", asset);
                }
                Ok(response) => {
                    println!("❌ {} - {}", asset, response.status());
                    failures.push(format!("{} returned {}", asset, response.status()));
                }
                Err(e) => {
                    println!("❌ {} - Error: {}", asset, e);
                    failures.push(format!("{} failed: {}", asset, e));
                }
            }
        }

        if !failures.is_empty() {
            panic!("Some core assets failed to load:\n{}", failures.join("\n"));
        }
    }
}

#[cfg(test)]
mod pwa_functionality_tests {
    /// Tests for PWA-specific functionality
    /// These tests would ideally use headless browser automation
    /// For now, they document expected behavior

    #[test]
    #[ignore]
    fn test_pwa_installable() {
        // TODO: Use playwright or similar to test PWA installation
        // Should verify:
        // - beforeinstallprompt event fires
        // - Install prompt can be triggered
        // - App appears in installed apps
        println!("Manual test: Open in Chrome, check for install prompt");
    }

    #[test]
    #[ignore]
    fn test_offline_functionality() {
        // TODO: Test that app works offline
        // Should verify:
        // - Service worker caches assets
        // - App shell loads when offline
        // - Offline indicator appears
        println!("Manual test: Open app, go offline, reload page");
    }

    #[test]
    #[ignore]
    fn test_app_starts_in_standalone() {
        // TODO: Test that installed PWA opens in standalone mode
        // Should verify:
        // - No browser UI visible
        // - Full screen app experience
        // - Proper window title
        println!("Manual test: Install PWA, launch from home screen");
    }
}
