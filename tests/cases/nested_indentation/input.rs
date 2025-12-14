mod my_module {
    struct MyStruct;

    impl MyStruct {
        #[instrument(level = "debug", target = LOG_TARGET, name = "nested-method", skip_all)]
        async fn nested_method(&self, param: String) -> Result<()> {
            Ok(())
        }

        #[test_macro(param1 = "value1", param2 = "value2", param3 = "value3", param4 = "value4")]
        fn another_nested(&self) {
            assert!(true);
        }
    }

    fn module_level() {
        #[instrument(level = "trace", target = LOG_TARGET, name = "inner-fn", skip_all)]
        fn inner_function() -> Result<()> {
            Ok(())
        }
    }
}
