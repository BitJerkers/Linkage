//! Utilities related to the firewall backends and their implementation.

/// Turns the supplied arguments into a Vec<String>. Converts the arguments using String::from.
#[macro_export]
macro_rules! to_string_vec {
    ( $( $x:expr ),* ) => {
        {
            vec![
                $(
                    String::from($x),
                )*
            ]
        }
    };
}

/// Executes the supplied expression for the supplied identifiers. Required a .execute method that
/// can accept the expression and returns a Result because it adds `?` to the method call.
#[macro_export]
macro_rules! executor_execute_for {
    ( $x:expr, $( $e:ident ),+ ) => {
        {
            $(
                $e.execute($x)?;
            )+
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::executor::{Executor, MockExecutor};
    use mockall::predicate::*;
    use crate::error::FirewallError;

    #[test]
    fn test_to_string_vec() {
        let v1: Vec<String> = vec![];
        let v2: Vec<String> = to_string_vec!();
        assert_eq!(
            v1,
            v2,
        );
        assert_eq!(
            vec![String::from("Hello")],
            to_string_vec!("Hello"),
        );
        assert_eq!(
            vec![String::from("Hello"), String::from("World")],
            to_string_vec!("Hello", "World"),
        );
        assert_eq!(
            vec![String::from("Hello"), String::from("World"), String::from("!!!")],
            to_string_vec!("Hello", "World", "!!!"),
        );
    }

    #[test]
    fn test_executor_execute_for() -> Result<(), FirewallError> {
        let mut e1_mock = MockExecutor::new();
        e1_mock.expect_execute()
            .times(1)
            .with(eq(to_string_vec!("hello", "world", "420")))
            .returning(|_| Ok(()));
        e1_mock.expect_execute()
            .times(1)
            .with(eq(to_string_vec!("test", "abc")))
            .returning(|_| Ok(()));

        let mut e2_mock = MockExecutor::new();
        e2_mock.expect_execute()
            .times(1)
            .with(eq(to_string_vec!("test", "abc")))
            .returning(|_| Ok(()));
        e2_mock.expect_execute()
            .times(1)
            .with(eq(to_string_vec!("cat")))
            .returning(|_| Err(FirewallError::IptablesError(None)));

        executor_execute_for!(to_string_vec!("hello", "world", "420"), e1_mock);
        executor_execute_for!(to_string_vec!("test", "abc"), e1_mock, e2_mock);

        // It should return the error if one occurred
        assert!(|| -> Result<(), FirewallError> {
            executor_execute_for!(to_string_vec!("cat"), e2_mock);
            Ok(())
        }().is_err());

        Ok(())
    }
}