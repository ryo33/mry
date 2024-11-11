#![allow(clippy::too_many_arguments)]

use mry::Any;

#[mry::mry]
#[derive(Default, PartialEq)]
struct Cat {
    name: String,
}

#[mry::mry]
impl Cat {
    fn meow(
        &self,
        count_part1: usize,
        count_part2: usize,
        count_part3: usize,
        count_part4: usize,
        count_part5: usize,
        count_part6: usize,
        count_part7: usize,
        count_part8: usize,
        count_part9: usize,
    ) -> String {
        format!(
            "{}: {}",
            self.name,
            "meow".repeat(
                count_part1
                    + count_part2
                    + count_part3
                    + count_part4
                    + count_part5
                    + count_part6
                    + count_part7
                    + count_part8
                    + count_part9
            )
        )
    }
}

#[test]
fn many_arguments() {
    let mut cat: Cat = mry::new!(Cat {
        name: "Tama".into(),
        ..Default::default()
    });
    let mock = cat
        .mock_meow(Any, Any, Any, Any, Any, Any, Any, Any, Any)
        .returns("something".to_string());
    assert_eq!(cat.meow(2, 2, 2, 2, 2, 2, 2, 2, 2), "something".to_string());
    mock.assert_called(1)
}
