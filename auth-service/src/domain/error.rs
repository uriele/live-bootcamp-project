pub enum AuthAPIError {
    InvalidCredentials,
    WrongEmailOrPassword,
    UserAlreadyExists,
    UserNotFound,
    InternalServerError,
    MissingToken,
    InvalidToken,
}