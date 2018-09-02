pub mod behavior_value;



#[cfg(test)]
mod tests {
    use behavior_value::BehaviorValue;

    #[test]
    fn debug_test() {
        println!("{:?}", BehaviorValue::Success::<bool, bool, bool>(true));
    }

    #[test]
    fn not_success_to_failure() {
        let n: BehaviorValue<i64, (), i64> = BehaviorValue::Success(12);
        assert!(
            match !n {
                BehaviorValue::Success(_) => false,
                BehaviorValue::Running(_) => false,
                BehaviorValue::Failure(n) => n == 12,
            }
        );
    }

    #[test]
    fn not_failure_to_success() {
        let n: BehaviorValue<f64, (), i64> = BehaviorValue::Failure(12);
        assert!(
            match !n {
                BehaviorValue::Success(n) => n == 12,
                BehaviorValue::Running(_) => false,
                BehaviorValue::Failure(_) => false,
            }
        );
    }
}
