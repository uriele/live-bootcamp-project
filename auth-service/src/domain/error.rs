pub enum AuthAPIErrors {
    InvalidCredentials,
    WrongEmailOrPassword,
    UserAlreadyExists,
    UserNotFound,
    InternalServerError,
    MissingToken,
    InvalidToken,
}