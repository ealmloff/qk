pub trait EventDescription<P: PlatformEvents> {
    type EventType;

    const ID: u16;

    const NAME: &'static str;

    const BUBBLES: bool = true;
}

pub trait PlatformEvents {
    type AnimationEvent;
    type BeforeUnloadEvent;
    type CompositionEvent;
    type DeviceMotionEvent;
    type DeviceOrientationEvent;
    type DragEvent;
    type ErrorEvent;
    type Event;
    type FocusEvent;
    type GamepadEvent;
    type HashChangeEvent;
    type InputEvent;
    type KeyboardEvent;
    type MessageEvent;
    type MouseEvent;
    type PageTransitionEvent;
    type PointerEvent;
    type PopStateEvent;
    type PromiseRejectionEvent;
    type SecurityPolicyViolationEvent;
    type StorageEvent;
    type SubmitEvent;
    type TouchEvent;
    type TransitionEvent;
    type UiEvent;
    type WheelEvent;
    type ProgressEvent;
}

impl<'a, P: PlatformEvents> PlatformEvents for &'a mut P {
    type AnimationEvent = P::AnimationEvent;
    type BeforeUnloadEvent = P::BeforeUnloadEvent;
    type CompositionEvent = P::CompositionEvent;
    type DeviceMotionEvent = P::DeviceMotionEvent;
    type DeviceOrientationEvent = P::DeviceOrientationEvent;
    type DragEvent = P::DragEvent;
    type ErrorEvent = P::ErrorEvent;
    type Event = P::Event;
    type FocusEvent = P::FocusEvent;
    type GamepadEvent = P::GamepadEvent;
    type HashChangeEvent = P::HashChangeEvent;
    type InputEvent = P::InputEvent;
    type KeyboardEvent = P::KeyboardEvent;
    type MessageEvent = P::MessageEvent;
    type MouseEvent = P::MouseEvent;
    type PageTransitionEvent = P::PageTransitionEvent;
    type PointerEvent = P::PointerEvent;
    type PopStateEvent = P::PopStateEvent;
    type PromiseRejectionEvent = P::PromiseRejectionEvent;
    type SecurityPolicyViolationEvent = P::SecurityPolicyViolationEvent;
    type StorageEvent = P::StorageEvent;
    type SubmitEvent = P::SubmitEvent;
    type TouchEvent = P::TouchEvent;
    type TransitionEvent = P::TransitionEvent;
    type UiEvent = P::UiEvent;
    type WheelEvent = P::WheelEvent;
    type ProgressEvent = P::ProgressEvent;
}

macro_rules! generate_event_types {
  {$(
    $( #[$no_bubble:ident] )?
    $event:ident : $event_type:ident
  ),* $(,)?} => {

    #[allow(dead_code, non_camel_case_types)]
    enum Events { $($event,)* __last}

    pub(crate) const EVENT_COUNT: usize = Events::__last as usize;

    $(
        #[doc = concat!("The [`", stringify!($event), "`](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/", stringify!($event), ") event, which receives ", stringify!($event_type), " as its argument.")]
        #[derive(Copy, Clone)]
        #[allow(non_camel_case_types)]
        pub struct $event;

        impl<P: PlatformEvents> EventDescription<P> for $event {
          type EventType = P::$event_type;

          const ID: u16 = Events::$event as u16;

          const NAME: &'static str = stringify!($event);

          $(
            generate_event_types!($no_bubble);
          )?
        }
    )*
  };

  (no_bubble) => {
    const BUBBLES: bool = false;
  }
}

generate_event_types! {
  #[no_bubble]
  afterprint: Event,
  #[no_bubble]
  beforeprint: Event,
  #[no_bubble]
  beforeunload: BeforeUnloadEvent,
  #[no_bubble]
  gamepadconnected: GamepadEvent,
  #[no_bubble]
  gamepaddisconnected: GamepadEvent,
  hashchange: HashChangeEvent,
  #[no_bubble]
  languagechange: Event,
  #[no_bubble]
  message: MessageEvent,
  #[no_bubble]
  messageerror: MessageEvent,
  #[no_bubble]
  offline: Event,
  #[no_bubble]
  online: Event,
  #[no_bubble]
  pagehide: PageTransitionEvent,
  #[no_bubble]
  pageshow: PageTransitionEvent,
  popstate: PopStateEvent,
  rejectionhandled: PromiseRejectionEvent,
  #[no_bubble]
  storage: StorageEvent,
  #[no_bubble]
  unhandledrejection: PromiseRejectionEvent,
  #[no_bubble]
  unload: Event,

  #[no_bubble]
  abort: UiEvent,
  animationcancel: AnimationEvent,
  animationend: AnimationEvent,
  animationiteration: AnimationEvent,
  animationstart: AnimationEvent,
  auxclick: MouseEvent,
  beforeinput: InputEvent,
  #[no_bubble]
  blur: FocusEvent,
  #[no_bubble]
  canplay: Event,
  #[no_bubble]
  canplaythrough: Event,
  change: Event,
  click: MouseEvent,
  #[no_bubble]
  close: Event,
  compositionend: CompositionEvent,
  compositionstart: CompositionEvent,
  compositionupdate: CompositionEvent,
  contextmenu: MouseEvent,
  #[no_bubble]
  cuechange: Event,
  doubleclick: MouseEvent,
  drag: DragEvent,
  dragend: DragEvent,
  dragenter: DragEvent,
  dragleave: DragEvent,
  dragover: DragEvent,
  dragexit: DragEvent,
  dragstart: DragEvent,
  drop: DragEvent,
  #[no_bubble]
  durationchange: Event,
  #[no_bubble]
  emptied: Event,
  #[no_bubble]
  encrypted: Event,
  #[no_bubble]
  ended: Event,
  #[no_bubble]
  error: ErrorEvent,
  #[no_bubble]
  focus: FocusEvent,
  #[no_bubble]
  focusin: FocusEvent,
  #[no_bubble]
  focusout: FocusEvent,
  formdata: Event,
  #[no_bubble]
  gotpointercapture: PointerEvent,
  input: Event,
  #[no_bubble]
  invalid: Event,
  keydown: KeyboardEvent,
  keypress: KeyboardEvent,
  keyup: KeyboardEvent,
  #[no_bubble]
  load: Event,
  #[no_bubble]
  loadeddata: Event,
  #[no_bubble]
  loadedmetadata: Event,
  #[no_bubble]
  loadstart: Event,
  lostpointercapture: PointerEvent,
  mousedown: MouseEvent,
  #[no_bubble]
  mouseenter: MouseEvent,
  #[no_bubble]
  mouseleave: MouseEvent,
  mousemove: MouseEvent,
  mouseout: MouseEvent,
  mouseover: MouseEvent,
  mouseup: MouseEvent,
  #[no_bubble]
  pause: Event,
  #[no_bubble]
  play: Event,
  #[no_bubble]
  playing: Event,
  pointercancel: PointerEvent,
  pointerdown: PointerEvent,
  #[no_bubble]
  pointerenter: PointerEvent,
  #[no_bubble]
  pointerleave: PointerEvent,
  pointermove: PointerEvent,
  pointerout: PointerEvent,
  pointerover: PointerEvent,
  pointerup: PointerEvent,
  #[no_bubble]
  progress: ProgressEvent,
  #[no_bubble]
  ratechange: Event,
  reset: Event,
  #[no_bubble]
  resize: UiEvent,
  #[no_bubble]
  scroll: Event,
  securitypolicyviolation: SecurityPolicyViolationEvent,
  #[no_bubble]
  seeked: Event,
  #[no_bubble]
  seeking: Event,
  select: Event,
  #[no_bubble]
  selectionchange: Event,
  selectstart: Event,
  slotchange: Event,
  #[no_bubble]
  stalled: Event,
  submit: SubmitEvent,
  #[no_bubble]
  suspend: Event,
  #[no_bubble]
  timeupdate: Event,
  #[no_bubble]
  toggle: Event,
  touchcancel: TouchEvent,
  touchend: TouchEvent,
  touchmove: TouchEvent,
  touchstart: TouchEvent,
  transitioncancel: TransitionEvent,
  transitionend: TransitionEvent,
  transitionrun: TransitionEvent,
  transitionstart: TransitionEvent,
  #[no_bubble]
  volumechange: Event,
  #[no_bubble]
  waiting: Event,
  webkitanimationend: Event,
  webkitanimationiteration: Event,
  webkitanimationstart: Event,
  webkittransitionend: Event,
  wheel: WheelEvent,

  DOMContentLoaded: Event,
  #[no_bubble]
  devicemotion: DeviceMotionEvent,
  #[no_bubble]
  deviceorientation: DeviceOrientationEvent,
  #[no_bubble]
  orientationchange: Event,

  copy: Event,
  cut: Event,
  paste: Event,

  fullscrehange: Event,
  fullscreenerror: Event,
  pointerlockchange: Event,
  pointerlockerror: Event,
  #[no_bubble]
  readystatechange: Event,
  visibilitychange: Event,
}
