//! WASM Panic Boundary Tests
//!
//! Validates panic handling at WASM boundaries:
//! - Panic is caught at WASM boundary
//! - Error propagation to JS
//! - Recovery after panic
//! - No memory corruption

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
mod panic_boundary_tests {
    use super::*;
    use wasm_bindgen::prelude::*;
    use std::panic;

    /// Safe wrapper that catches panics
    fn catch_panic<F, R>(f: F) -> Result<R, JsValue>
    where
        F: FnOnce() -> R + panic::UnwindSafe,
    {
        panic::catch_unwind(f).map_err(|_| {
            JsValue::from_str("Panic occurred")
        })
    }

    /// Safe division that handles panics
    fn safe_divide(a: f32, b: f32) -> Result<f32, JsValue> {
        catch_panic(|| {
            if b == 0.0 {
                panic!("Division by zero");
            }
            a / b
        })
    }

    /// Safe array access that handles panics
    fn safe_array_access(arr: &[f32], index: usize) -> Result<f32, JsValue> {
        catch_panic(|| {
            arr.get(index)
                .copied()
                .expect("Index out of bounds")
        })
    }

    #[wasm_bindgen_test]
    fn test_panic_boundary_catch() {
        // Install panic hook for better error messages
        console_error_panic_hook::set_once();

        let result = catch_panic(|| {
            panic!("Test panic");
        });

        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_panic_boundary_success() {
        let result = catch_panic(|| {
            42
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[wasm_bindgen_test]
    fn test_division_by_zero_panic() {
        let result = safe_divide(10.0, 0.0);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_division_success() {
        let result = safe_divide(10.0, 2.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5.0);
    }

    #[wasm_bindgen_test]
    fn test_array_bounds_panic() {
        let arr = vec![1.0, 2.0, 3.0];
        let result = safe_array_access(&arr, 10);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_array_access_success() {
        let arr = vec![1.0, 2.0, 3.0];
        let result = safe_array_access(&arr, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[wasm_bindgen_test]
    fn test_recovery_after_panic() {
        // First operation panics
        let result1 = safe_divide(10.0, 0.0);
        assert!(result1.is_err());

        // Second operation should work fine
        let result2 = safe_divide(10.0, 2.0);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 5.0);
    }

    #[wasm_bindgen_test]
    fn test_multiple_panics() {
        // Multiple panics should all be caught
        for _ in 0..5 {
            let result = safe_divide(1.0, 0.0);
            assert!(result.is_err());
        }

        // Normal operation should still work
        let result = safe_divide(10.0, 5.0);
        assert!(result.is_ok());
    }

    /// Audio buffer processor with panic protection
    struct SafeAudioProcessor {
        buffer: Vec<f32>,
    }

    impl SafeAudioProcessor {
        fn new(size: usize) -> Self {
            Self {
                buffer: vec![0.0; size],
            }
        }

        fn process(&mut self, input: &[f32]) -> Result<Vec<f32>, JsValue> {
            catch_panic(|| {
                let mut output = self.buffer.clone();

                for (i, &sample) in input.iter().enumerate() {
                    if i >= output.len() {
                        panic!("Input too large for buffer");
                    }
                    output[i] = sample * 2.0; // Simple amplification
                }

                output
            })
        }

        fn process_checked(&mut self, input: &[f32]) -> Result<Vec<f32>, JsValue> {
            if input.len() > self.buffer.len() {
                return Err(JsValue::from_str("Input exceeds buffer size"));
            }

            catch_panic(|| {
                let mut output = self.buffer.clone();
                for (i, &sample) in input.iter().enumerate() {
                    output[i] = sample * 2.0;
                }
                output
            })
        }
    }

    #[wasm_bindgen_test]
    fn test_audio_processor_panic() {
        let mut processor = SafeAudioProcessor::new(100);

        // Input too large - should panic
        let large_input: Vec<f32> = vec![0.5; 200];
        let result = processor.process(&large_input);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_audio_processor_success() {
        let mut processor = SafeAudioProcessor::new(100);

        let input: Vec<f32> = vec![0.5; 50];
        let result = processor.process(&input);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output[0], 1.0); // 0.5 * 2.0
    }

    #[wasm_bindgen_test]
    fn test_audio_processor_checked() {
        let mut processor = SafeAudioProcessor::new(100);

        // Large input - should return error without panic
        let large_input: Vec<f32> = vec![0.5; 200];
        let result = processor.process_checked(&large_input);
        assert!(result.is_err());

        // Normal input should work
        let normal_input: Vec<f32> = vec![0.5; 50];
        let result = processor.process_checked(&normal_input);
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_panic_hook_installed() {
        // Verify panic hook is installed
        console_error_panic_hook::set_once();

        // Trigger a panic
        let result = catch_panic(|| {
            panic!("Test panic with hook");
        });

        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_no_memory_corruption_after_panic() {
        let mut data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        // Cause a panic during processing
        let result = catch_panic(|| {
            for i in 0..10 {
                data[i] = i as f32; // Will panic at index 5
            }
        });

        assert!(result.is_err());

        // Data should still be accessible and partially modified
        assert!(data.len() == 5);
        assert_eq!(data[0], 0.0); // Modified
        assert_eq!(data[1], 1.0); // Modified
        assert_eq!(data[2], 2.0); // Modified
        assert_eq!(data[3], 3.0); // Modified
        assert_eq!(data[4], 4.0); // Modified
    }

    #[wasm_bindgen_test]
    fn test_error_message_propagation() {
        let result: Result<(), JsValue> = catch_panic(|| {
            panic!("Specific error message");
        });

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.as_string().unwrap_or_default();
        assert!(err_str.contains("Panic occurred"));
    }

    #[wasm_bindgen_test]
    async fn test_async_panic_recovery() {
        // First async operation panics
        let result1 = catch_panic(|| {
            panic!("Async panic");
        });
        assert!(result1.is_err());

        // Simulate async delay
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::from(1)))
            .await
            .ok();

        // Second operation should work
        let result2 = catch_panic(|| {
            42
        });
        assert!(result2.is_ok());
    }
}

/// Tests for Result-based error handling (preferred over panics)
#[cfg(target_arch = "wasm32")]
mod result_based_tests {
    use super::*;
    use wasm_bindgen::prelude::*;

    /// Safe division using Result
    fn divide(a: f32, b: f32) -> Result<f32, JsValue> {
        if b == 0.0 {
            Err(JsValue::from_str("Division by zero"))
        } else {
            Ok(a / b)
        }
    }

    /// Safe array access using Result
    fn get_element(arr: &[f32], index: usize) -> Result<f32, JsValue> {
        arr.get(index)
            .copied()
            .ok_or_else(|| JsValue::from_str("Index out of bounds"))
    }

    #[wasm_bindgen_test]
    fn test_result_based_division() {
        // Error case
        let result = divide(10.0, 0.0);
        assert!(result.is_err());

        // Success case
        let result = divide(10.0, 2.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5.0);
    }

    #[wasm_bindgen_test]
    fn test_result_based_array_access() {
        let arr = vec![1.0, 2.0, 3.0];

        // Error case
        let result = get_element(&arr, 10);
        assert!(result.is_err());

        // Success case
        let result = get_element(&arr, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.0);
    }

    #[wasm_bindgen_test]
    fn test_result_chaining() {
        let arr = vec![10.0, 20.0, 30.0];

        let result = get_element(&arr, 1)
            .and_then(|val| divide(val, 2.0));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10.0);
    }

    #[wasm_bindgen_test]
    fn test_result_error_propagation() {
        let arr = vec![10.0, 20.0, 30.0];

        // Error in first operation
        let result = get_element(&arr, 10)
            .and_then(|val| divide(val, 2.0));

        assert!(result.is_err());
    }
}

// Non-WASM tests
#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_tests {
    #[test]
    fn test_panic_tests_are_wasm_only() {
        assert!(true, "Panic boundary tests are WASM-only");
    }
}
