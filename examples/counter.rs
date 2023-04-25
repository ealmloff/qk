use qk::prelude::*;

#[component]
fn Counter(cx: Scope) {
    // Rx makes the variable reactive.
    // Reactive variables can be used in rsx and reactive closures as if they were Copy.
    // Reactive variables implemet Deref and DerefMut.
    let num: Rx<i32> = 0;

    rsx! {
        // You can attach closures to events.
        <button onclick=|_| *num += 1>
            "increase"
        </button>
        <button onclick=|_| *num -= 1>
            "decrease"
        </button>
        <div>
            // Text in rsx can be formatted with {}
            "count: {num}"
        </div>
    }
}

fn main() {
    let ui = WebRenderer::default();
    launch(ui, Counter {});
}
