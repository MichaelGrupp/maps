fn main() -> eframe::Result {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use maps::main_native::main_native;
        main_native()
    }

    #[cfg(target_arch = "wasm32")]
    {
        use maps::main_wasm::main_wasm;
        main_wasm();
        Ok(())
    }
}
