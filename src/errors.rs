// Jianing Yang <jianingy.yang@gmail.com> @  2 Oct, 2016
error_chain!{

    errors {
        UartError(reason: String) {
            description("uart port error")
            display("uart port error: {}", reason)
        }

        DataError(reason: String) {
            description("data error")
            display("data error: {}", reason)
        }

        DatabaseError(reason: String) {
            description("database error")
            display("database error: {}", reason)
        }

        PruCodeError(reason: String) {
            description("PRU code error")
            display("PRU code error: {}", reason)
        }
    }

}

#[macro_export]
macro_rules! uart_error {
    ( $( $e:expr ),* ) => {
        ErrorKind::UartError(format!($( $e ),*))
    }
}

#[macro_export]
macro_rules! data_error {
    ( $( $e:expr ),* ) => {
        ErrorKind::DataError(format!($( $e ),*))
    }
}

#[macro_export]
macro_rules! db_error {
    ( $( $e:expr ),* ) => {
        ErrorKind::DatabaseError(format!($( $e ),*))
    }
}
