use tauri::command;

/// Attempt biometric (Touch ID) or device authentication on macOS.
/// Returns "ok" on success, "failed" on auth failure, "unavailable" if biometrics not supported.
/// On non-macOS returns "unavailable".
#[command]
pub fn try_biometric_auth() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        // Write a small Swift program that calls LocalAuthentication framework.
        // Swift is available on any Mac with Xcode Command Line Tools.
        let script = r#"import Foundation
import LocalAuthentication

let ctx = LAContext()
var err: NSError?

guard ctx.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &err) else {
    print("unavailable")
    exit(0)
}

let sema = DispatchSemaphore(value: 0)
var ok = false

ctx.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: "Unlock Ganymede") { success, _ in
    ok = success
    sema.signal()
}
_ = sema.wait(timeout: .distantFuture)
print(ok ? "ok" : "failed")
"#;

        let tmp = std::env::temp_dir().join("ganymede_bio.swift");
        std::fs::write(&tmp, script).map_err(|e| e.to_string())?;

        let output = std::process::Command::new("swift")
            .arg(tmp.to_str().unwrap_or(""))
            .output()
            .map_err(|e| format!("unavailable: {}", e))?;

        std::fs::remove_file(&tmp).ok();

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(if stdout.is_empty() {
            "unavailable".to_string()
        } else {
            stdout
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok("unavailable".to_string())
    }
}
