// The functions are the same except for their names
// I created a macro to generate them
// the + $(,)? adds an optional comma between the names
#[macro_export]
macro_rules! make_response_functions{
    ($($name:ident),+ $(,)?) => {
        $(pub async fn $name() -> axum::http::StatusCode {
            axum::http::StatusCode::OK
        })*
    }
}