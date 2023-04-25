use qk::prelude::*;

#[component]
fn List(cx: Scope) {
    let nums: Rx<Vec<i32>> = vec![];

    rsx! {
        <button onclick=|_| {
            nums.push(0);
            for num in &mut *nums {
                *num += 1;
            }
        }>"more!"</button>
        <button onclick=|_| {
            nums.clear()
        }>"clear!"</button>
        <div>
            "nums={nums:?}"
        </div>
    }
}

fn main() {
    let ui = WebRenderer::default();
    launch(ui, List {});
}
