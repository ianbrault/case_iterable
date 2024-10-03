/*
** tests/test_macro.rs
*/

use case_iterable::CaseIterable;

#[test]
fn test_expand() {
    #[derive(CaseIterable, Debug, PartialEq)]
    enum TestEnum {
        A = 0,
        B,
        C,
    }

    let cases = TestEnum::all_cases().collect::<Vec<_>>();
    assert_eq!(cases.len(), 3);
    assert_eq!(cases[0], TestEnum::A);
    assert_eq!(cases[1], TestEnum::B);
    assert_eq!(cases[2], TestEnum::C);
}
