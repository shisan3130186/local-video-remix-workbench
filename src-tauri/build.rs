fn main() {
    println!(
        "cargo:rustc-env=SMARTCUT_MEMBERSHIP_API_URL=https://smartcut-membership-d8bfdff54769-1457945645.ap-shanghai.app.tcloudbase.com/smartcut-membership"
    );
    println!(
        "cargo:rustc-env=SMARTCUT_MEMBERSHIP_PUBLIC_KEY_BASE64=MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAECdFy1q1LGnO/nIGT/cUqMHl3srlDYLpRRGWCEP1G706E1T8vyZXP8cLkQrsZArwBqWy6gUdRSNMiWJKVXqJCWg=="
    );
    println!("cargo:rerun-if-changed=build.rs");
    tauri_build::build()
}
