use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Rect, Role, Tree, TreeId, TreeUpdate,
};
use accesskit_rs::SubclassingAdapter;
use appkit::prelude::*;

const WINDOW: NodeId = NodeId(0);
const BUTTON: NodeId = NodeId(1);
const LABEL: NodeId = NodeId(2);

struct InitialTree;

impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        let mut window = Node::new(Role::Window);
        window.set_children(vec![BUTTON, LABEL]);
        let mut button = Node::new(Role::Button);
        button.set_label("Press me");
        button.set_bounds(Rect::new(40.0, 40.0, 360.0, 120.0));
        button.add_action(Action::Click);
        button.add_action(Action::Focus);
        let mut label = Node::new(Role::Label);
        label.set_value("Hello from accesskit-rs");
        label.set_bounds(Rect::new(40.0, 160.0, 760.0, 220.0));
        Some(TreeUpdate {
            nodes: vec![(WINDOW, window), (BUTTON, button), (LABEL, label)],
            tree: Some(Tree::new(WINDOW)),
            tree_id: TreeId::ROOT,
            focus: BUTTON,
        })
    }
}

struct PrintActions;

impl ActionHandler for PrintActions {
    fn do_action(&mut self, request: ActionRequest) {
        println!("{:?} on {:?}", request.action, request.target_node);
    }
}

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("an example starts on the main thread");
    let application = Application::shared(main_thread);
    assert!(application.set_activation_policy(ActivationPolicy::Regular));
    let window = Window::new(main_thread, CGRect::new(200.0, 200.0, 480.0, 240.0), "accesskit-rs")?;
    let _adapter = unsafe { SubclassingAdapter::for_window(window.as_ptr(), InitialTree, PrintActions) };
    window.make_key_and_order_front();
    application.activate();
    println!("The window is open. Press Control-C to stop.");
    application.run();
    Ok(())
}
