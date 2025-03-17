mod async_fn_in_trait;
mod async_fn_trait_variant;
mod async_method;
mod async_trait;
mod bounds;
mod complex_clone;
mod function_style_macro;
mod generics;
mod impl_trait;
mod iterator;
mod many_arguments;
mod mock_trait;
mod mut_param;
mod nested_mock;
mod not_clone;
mod partial_mock;
mod reference_and_pattern;
mod returns_with_recursive_call;
mod simple_case;
mod skip_arg;
mod skip_fns;
mod static_function;

#[cfg(feature = "send_wrapper")]
mod non_send;
#[cfg(feature = "send_wrapper")]
mod raw_pointer;
