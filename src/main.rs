pub mod copy;
// pub mod copy_arena;
pub mod copy_ll;
pub mod reactive;
mod renderer;
pub mod signal;
mod web;

use qk_macro::rsx;
use renderer::Renderer;
use std::fmt::Debug;
use std::fmt::Formatter;
use wasm_bindgen::prelude::*;
use web::WebRenderer;

use crate::copy::*;

#[derive(Debug, Clone)]
struct RootNodes(Box<[u32]>);

#[wasm_bindgen(inline_js = r#"
export function clear_body() {
    document.body.innerHTML = "";
}
"#)]
extern "C" {
    fn clear_body();
}

fn main() {
    let runtime = claim_rt();
    let mut ui = WebRenderer::default();
    for i in 0..100 {
        {
            let mut scope = scope!(runtime);
            let (x, nodes) = Benchmark::create_with(&mut scope, &mut ui, |_| {
                (0..10000)
                    .map(|id| RowData {
                        id,
                        label: "inexpensive yellow table".to_string(),
                    })
                    .collect()
            });
            ui.append_all(0, nodes.0.iter().copied());
            ui.flush();
            
            fn remove(x: State<Benchmark>, ui: &mut impl Renderer) {
                x.modify(|x| x.remove(ui));
                ui.flush();
            }
            if i != 99 {
                remove(x, &mut ui);
            }
        }
        // for i in 0..1000 {
        //     web_sys::console::log_1(&"delay".into());
        // }
        // clear_body();
    }
}

#[derive(Clone, Debug)]
struct RowData {
    id: usize,
    label: String,
}

#[derive(Clone, Debug)]
// Subscriptions are confined to components, memos are shared between components
struct Benchmark {
    /*=*/ rows: Vec<State<Row>>,
    selected_row: Option<usize>,
    dyn_nodes: [u32; 1],
    // construct(rows: Vec<RowData>) => {
    //     rows = rowsRow {
    //         label: row.label,
    //         selected <= self.selected_row() == Some(row.id),
    //         select: || *self.selected_row_mut() = Some(row.id),
    //         remove: || self.rows_mut().remove(row.id),
    //     }
    //     selected_row = None;
    // }
    // view(rows: Vec<RowData>) <= {
    //     div {
    //         ...self.rows()
    //     }
    // }
}

impl Benchmark {
    
    fn create_with(
        scope: &Scope,
        ui: &mut impl Renderer,
        f: impl FnOnce(State<Self>) -> Vec<RowData>,
    ) -> (State<Self>, RootNodes) {
        let mut dyn_nodes = [0; 1];
        let mut roots = [0; 1];

        dyn_nodes[0] = ui.node();
        roots[0] = dyn_nodes[0];
        ui.create_element(dyn_nodes[0], "div");

        let myself = scope.state_with(|myself: State<Self>| {
            let rows = f(myself);
            let mut _rows = Vec::with_capacity(rows.len());
            let mut _children = Vec::with_capacity(rows.len());
            for row in rows {
                let (_row, _row_nodes) = Row::create_with(scope, ui, move |_row| {
                    let row = row;
                    let selected = scope.state(None == Some(row.id));
                    let select = Box::new(move || {
                        myself.modify(|myself| {
                            _row.with(|_row| {
                                *myself.selected_row_mut() = Some(row.id);
                            });
                        })
                    });
                    let remove = Box::new(move || {
                        myself.modify(|myself| {
                            _row.with(|_row| {
                                myself.rows_mut().remove(row.id);
                            });
                        })
                    });
                    (row, selected, select, remove)
                });

                _rows.push(_row);
                _children.push(_row_nodes);
            }

            ui.append_all(
                dyn_nodes[0],
                _children.iter().flat_map(|x| x.0.iter()).copied(),
            );

            Self {
                selected_row: None,
                rows: _rows,
                dyn_nodes,
            }
        });

        (myself, RootNodes(roots.into()))
    }

    fn remove(&mut self, ui: &mut impl Renderer) {
        for child in &self.rows {
            child.modify(|child| child.remove(ui));
        }
        ui.remove(self.dyn_nodes[0]);
        for node in &self.dyn_nodes {
            ui.return_node(*node);
        }
    }

    #[inline]
    fn selected_row(&self) -> Option<usize> {
        self.selected_row
    }

    #[inline]
    fn update_selected(&mut self) {
        let Self {
            rows, selected_row, ..
        } = self;
        for row in rows {
            row.modify(|row| row.selected.set(*selected_row == Some(row.data().id)));
        }
    }

    #[inline]
    fn update_rows(&mut self) {
        // TODO: keyed diffing
        self.rows.clear();
    }

    #[inline]
    fn selected_row_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_row
    }

    #[inline]
    fn rows_mut(&mut self) -> &mut Vec<State<Row>> {
        &mut self.rows
    }
}

struct Row {
    /*=*/ data: RowData,
    /*=>*/ selected: State<bool>,
    /*=*/ select: Box<dyn FnMut()>,
    /*=*/ remove: Box<dyn FnMut()>,
    dyn_nodes: [u32; 3],
    // {
    // id 1
    //   tr { class: selected.then(|| "danger").unwrap_or_default(),
    // id 2
    //      td { class:"col-md-1", "{id}" }
    //          td { class:"col-md-4", onclick: move |_| select(),
    // id 3
    //              a { class: "lbl", "{label}" }
    //          }
    //          td { class: "col-md-1",
    //              a { class: "remove", onclick: move |_| remove(),
    //                  span { class: "glyphicon glyphicon-remove remove", aria_hidden: "true" }
    //              }
    //          }
    //          td { class: "col-md-6" }
    //      }
    // }
}

impl Debug for Row {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Row")
            .field("data", &self.data)
            .field("selected", &self.selected)
            .finish()
    }
}

impl Row {
    #[inline]
    fn create_with(
        scope: &Scope,
        ui: &mut impl Renderer,
        f: impl FnOnce(State<Self>) -> (RowData, State<bool>, Box<dyn FnMut()>, Box<dyn FnMut()>),
    ) -> (State<Self>, RootNodes) {
        let cl = |scope: &Scope| {
            let mut roots = [0; 1];
            let myself = scope.state_with(|myself: State<Self>| {
                let (data, selected, select, remove) = f(myself);
                
                rsx! {
                    <tr danger={if true {"danger"} else {""}} >
                        <td class="col-md-1">r#"{&format!("{}", data.id)}"#</td>
                        <td class="col-md-4">
                            <a class="lbl">r#"{&data.label}"#</a>
                        </td>
                        <td class="col-md-1">
                            <a class="remove">
                                <span class="glyphicon glyphicon-remove remove" aria-hidden="true"></span>
                            </a>
                        </td>
                        <td class="col-md-6"></td>
                    </tr>
                }

                let dyn_nodes = [__dyn_n_0, __dyn_n_1, __dyn_n_2];

                Self {
                    data,
                    selected,
                    select,
                    remove,
                    dyn_nodes,
                }
            });
            (myself, RootNodes(roots.into()))
        };
        child_scope!(scope, cl)
    }

    fn remove(&mut self, ui: &mut impl Renderer) {
        ui.remove(self.dyn_nodes[0]);
        for dyn_node in self.dyn_nodes.iter() {
            ui.return_node(*dyn_node);
        }
    }

    #[inline]
    fn data(&self) -> &RowData {
        &self.data
    }
}
