use State;

#[test]
fn test_state_init() {
    let mut _s = State::new();
}

#[test]
#[should_fail]
fn test_error() {
    let mut s = State::new();
    s.pushinteger(42);
    s.error()
}

#[test]
fn test_errorstr() {
    let res = do ::std::task::try::<()> {
        let mut s = State::new();
        s.errorstr("some err");
    };
    let err = res.unwrap_err();
    let expected = "unprotected error in call to Lua API (some err)";
    let s = err.as_ref::<~str>();
    if s.is_some() {
        assert_eq!(s.unwrap().as_slice(), expected);
    } else {
        let s = err.as_ref::<&'static str>();
        if s.is_some() {
            assert_eq!(*s.unwrap(), expected);
        } else {
            fail!("unexpected failure result");
        }
    }
}
