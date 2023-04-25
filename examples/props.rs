use qk::prelude::*;

#[component]
fn TakesProp(cx: Scope, number: i32) {
    rsx! {
        <div>
            "{number}"
        </div>
    }
}

fn main() {
    let ui = WebRenderer::default();
    launch(
        ui,
        TakesProp {
            // The component takes a prop called `number` of type `i32`.
            number: 42,
        },
    );
}
