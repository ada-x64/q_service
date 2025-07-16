/// Creates aliases for the commonly used service types.
/// ## Parameters:
/// Example: The service's name. This is expected to be a plain type label
/// passed in. The macro will create the necessary structs for you.
///
/// $d: The data type. It must implement
/// [ServiceData](crate::prelude::ServiceData).
///
/// $e: The error type. It must implement
/// [ServiceError](crate::prelude::ServiceError).
///
/// ## Example usage
/// ```rust, ignore
/// use q_service::prelude::*;
/// use bevy::prelude::*;
///
/// #[derive(ServiceData, Clone, Default, PartialEq, Debug)]
/// struct MyData;
/// #[derive(ServiceError, thiserror::Error, PartialEq, Debug, Clone)]
/// enum MyError {}
///
/// service!(Example, MyData, MyError);
///
/// let app = App::new();
/// app.add_service(ExampleService::spec());
/// ```
///
/// ## Example output
/// ```rust, skip
/// /// Label for the state. Works as part of a unique identifier.
/// #[derive(ServiceLabel, PartialEq, Eq, Debug, Copy, Clone, Hash)]
/// pub struct ExampleLabel;
/// pub type ExampleSpec = ServiceSpec<ExampleLabel, ExampleData, ExampleError>;
/// pub type Example = Service<ExampleLabel, ExampleData, ExampleError>;
/// pub type ExampleHooks = ServiceHooks<ExampleError>;
/// /// Track service state changes. Inner value is a tuple, (previous_state, current_state).
/// pub type ExampleStateChange = ServiceStateChange<ExampleLabel, ExampleData, ExampleError>;
/// pub type Enter ExampleState = EnterServiceState<ExampleLabel, ExampleData, ExampleError>;
/// pub type Exit ExampleState = ExitServiceState<ExampleLabel, ExampleData, ExampleError>;
/// pub type ExampleEnabled = ServiceEnabled<ExampleLabel, ExampleData, ExampleError>;
/// pub type ExampleDisabled = ServiceDisabled<ExampleLabel, ExampleData, ExampleError>;
/// pub type ExampleInitialized = ServiceInitialized<ExampleLabel, ExampleData, ExampleError>;
/// pub type ExampleFailed = ServiceFailed<ExampleLabel, ExampleData, ExampleError>;
/// ```
#[macro_export]
macro_rules! service {
    ($t:ty, $d:ty, $e:ty) => {
        $crate::paste::paste! {
            mod [<$t:lower:snake _alias_impl>] {
                use super::*;
                use $crate::prelude::*;
                use std::marker::PhantomData;
                /// Label for the state. Works as part of a unique identifier.
                #[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
                pub struct [<$t Label>];
                impl ServiceLabel for [<$t Label>] {}
                pub type [<$t Spec>]= ServiceSpec<[<$t Label>], $d, $e>;
                pub type [<$t>] = Service<[<$t Label>], $d, $e>;
                pub type [<$t Hooks>] = ServiceHooks<$e>;
                /// Track service state changes. Inner value is a tuple, (previous_state, current_state).
                pub type [<$t StateChange>] = ServiceStateChange<[<$t Label>], $d, $e>;
                pub type [<Enter $t State>] = EnterServiceState<[<$t Label>], $d, $e>;
                pub type [<Exit $t State>] = ExitServiceState<[<$t Label>], $d, $e>;
                pub type [<$t Enabled>] = ServiceEnabled<[<$t Label>], $d, $e>;
                pub type [<$t Disabled>] = ServiceDisabled<[<$t Label>], $d, $e>;
                pub type [<$t Initialized>] = ServiceInitialized<[<$t Label>], $d, $e>;
                pub type [<$t Failed>] = ServiceFailed<[<$t Label>], $d, $e>;
            }
            pub use [< $t:lower _alias_impl >]::*;
        }
    };
}
