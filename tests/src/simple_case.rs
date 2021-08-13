use mry::Mock;

#[derive(Default)]
struct Cat {
    name: String,
    // TODO: auto generate
    // #[cfg(test)]
    meow: Mock<usize, String>,
}

impl Cat {
    fn meow(&self, count: usize) -> String {
        // TODO: auto generate
        #[cfg(test)]
        return self.meow._inner_behave(count);

        "meow".repeat(count)
    }
}

#[test]
fn meow_acts() {
    let mut cat = Cat {
        name: "Tama".into(),
        meow: Default::default(),
    };
    cat.meow.behaves(|count| format!("Called with {}", count));

    assert_eq!(cat.meow(2), "Called with 2".to_string());
}

#[test]
fn assert_called_with_meow_acts() {
    let mut cat = Cat {
        name: "Tama".into(),
        meow: Default::default(),
    };
    cat.meow.behaves(|count| format!("Called with {}", count));

    cat.meow(2);

    cat.meow.assert_called_with(2);
}

#[test]
#[should_panic]
fn assert_called_with_meow_acts_fails() {
    let mut cat = Cat {
        name: "Tama".into(),
        meow: Default::default(),
    };
    cat.meow.behaves(|count| format!("Called with {}", count));

    cat.meow(3);

    cat.meow.assert_called_with(2);
}

// #[test]
// fn meow_acts_when() {
// 	let mut cat = Cat {
//         name: "Tama".into(),
//         meow: Default::default(),
// 	};
// 	cat.meow.acts_when(eq(3), |count| {
//         format!("Called with {}", count)
//     });

// 	assert_eq!(cat.meow(3), "Called with 3".to_string())
// }
