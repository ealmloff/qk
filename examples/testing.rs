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

fn main() {}
