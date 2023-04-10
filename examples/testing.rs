// // Fine grained reactive
// // Copy state
// // United Client + Server => Server side code with client optimistic updates
// // Struct based with reactive fields
//
// #[reactive]
// struct Benchmark {
//     rows: Vec<Row>,
//     selected_row: Option<usize>,
//
//     view <= {
//         div {
//             for row in rows {
//                 Row {
//                     label: row.label,
//                     selected => selected_row() == Some(row.id),
//                     select: || selected_row.set(Some(row.id)),
//                     remove: || rows.modify(|r| r.remove(row.id)),
//                 }
//             }
//         }
//     }
// }
//
// #[reactive]
// struct Row {
//     label: String,
//     #[raw]
//     id: usize,
//     selected: bool,
//     select: impl FnMut(),
//     remove: impl FnMut(),
//
//     {
//       tr { class: selected.then(|| "danger").unwrap_or_default(),
//          td { class:"col-md-1", "{id}" }
//              td { class:"col-md-4", onclick: move |_| select(),
//                  a { class: "lbl", "{label}" }
//              }
//              td { class: "col-md-1",
//                  a { class: "remove", onclick: move |_| remove(),
//                      span { class: "glyphicon glyphicon-remove remove", aria_hidden: "true" }
//                  }
//              }
//              td { class: "col-md-6" }
//          }
//     }
// }

use qk::*;
use qk_macro::component;

fn main() {
    #[component]
    fn Foo(cx: Scope) {
        let x: Rx<i32> = 0;
        let y: Rx<i32> = 0;
        let z: Rx<Vec<i32>> = vec![];
        let w = 0;
        rx(move || {
            let x = *x;
            println!("{x} {w}");
        });
        rx(move || {
            let x = *x;
            let y = *y;
            println!("{x}");
        });
        rx(move || {
            let x = *x;
            let y = *y;
            let z = &*z;
            let mut w = z.clone();
            w.push(x);
            println!("{w:?}");
        });

        x += 1;
    }
    println!("with_x");
    comp.with_x(|mut x| {
        *x = *x + 1;
    });
    println!("with_z");
    comp.with_z(|mut z| {
        z.push(1);
    });
}
