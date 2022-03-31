/// Rappresent all possible js events used in reView
#[derive(strum_macros::AsRefStr, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum EventType {
    #[strum(serialize = "abort")]
    OnAbort,
    #[strum(serialize = "animationcancel")]
    OnAnimationCancel,
    #[strum(serialize = "animationend")]
    OnAnimationEnd,
    #[strum(serialize = "animationiteration")]
    OnAnimationIteration,
    #[strum(serialize = "animationstart")]
    OnAnimationStart,
    #[strum(serialize = "auxclick")]
    OnAuxClick,
    #[strum(serialize = "blur")]
    OnBlur,
    #[strum(serialize = "canplay")]
    OnCanPlay,
    #[strum(serialize = "canplaythrough")]
    OnCanPlaythrough,
    #[strum(serialize = "change")]
    OnChange,
    #[strum(serialize = "click")]
    OnClick,
    #[strum(serialize = "close")]
    OnClose,
    #[strum(serialize = "contextmenu")]
    OnContextMenu,
    #[strum(serialize = "copy")]
    OnCopy,
    #[strum(serialize = "cut")]
    OnCut,
    #[strum(serialize = "dblclick")]
    OnDblClick,
    #[strum(serialize = "drag")]
    OnDrag,
    #[strum(serialize = "dragend")]
    OnDragEnd,
    #[strum(serialize = "dragenter")]
    OnDragEnter,
    #[strum(serialize = "dragexit")]
    OnDragExit,
    #[strum(serialize = "dragleave")]
    OnDragLeave,
    #[strum(serialize = "dragover")]
    OnDragOver,
    #[strum(serialize = "dragstart")]
    OnDragStart,
    #[strum(serialize = "drop")]
    OnDrop,
    #[strum(serialize = "durationchange")]
    OnDurationChange,
    #[strum(serialize = "emptied")]
    OnEmptied,
    #[strum(serialize = "ended")]
    OnEnded,
    #[strum(serialize = "error")]
    OnError,
    #[strum(serialize = "focus")]
    OnFocus,
    #[strum(serialize = "gotpointercapture")]
    OnGotPointerCapture,
    #[strum(serialize = "input")]
    OnInput,
    #[strum(serialize = "invalid")]
    OnInvalid,
    #[strum(serialize = "keydown")]
    OnKeyDown,
    #[strum(serialize = "keypress")]
    OnKeyPress,
    #[strum(serialize = "keyup")]
    OnKeyUp,
    #[strum(serialize = "load")]
    OnLoad,
    #[strum(serialize = "loadeddata")]
    OnLoadedData,
    #[strum(serialize = "loadedmetadata")]
    OnLoadedMetadata,
    #[strum(serialize = "loadend")]
    OnLoadEnd,
    #[strum(serialize = "loadstart")]
    OnLoadStart,
    #[strum(serialize = "lostpointercapture")]
    OnLostPointerCapture,
    #[strum(serialize = "mousedown")]
    OnMouseDown,
    #[strum(serialize = "mouseenter")]
    OnMouseEnter,
    #[strum(serialize = "mouseleave")]
    OnMouseLeave,
    #[strum(serialize = "mousemove")]
    OnMouseMove,
    #[strum(serialize = "mouseout")]
    OnMouseOut,
    #[strum(serialize = "mouseover")]
    OnMouseOver,
    #[strum(serialize = "mouseup")]
    OnMouseUp,
    #[strum(serialize = "paste")]
    OnPaste,
    #[strum(serialize = "pause")]
    OnPause,
    #[strum(serialize = "play")]
    OnPlay,
    #[strum(serialize = "playing")]
    OnPlaying,
    #[strum(serialize = "pointercancel")]
    OnPointerCancel,
    #[strum(serialize = "pointerdown")]
    OnPointerDown,
    #[strum(serialize = "pointerenter")]
    OnPointerEnter,
    #[strum(serialize = "pointerleave")]
    OnPointerLeave,
    #[strum(serialize = "pointermove")]
    OnPointerMove,
    #[strum(serialize = "pointerout")]
    OnPointerOut,
    #[strum(serialize = "pointerover")]
    OnPointerOver,
    #[strum(serialize = "pointerup")]
    OnPointerUp,
    #[strum(serialize = "progress")]
    OnProgress,
    #[strum(serialize = "ratechange")]
    OnRateChange,
    #[strum(serialize = "reset")]
    OnReset,
    #[strum(serialize = "resize")]
    OnResize,
    #[strum(serialize = "scroll")]
    OnScroll,
    #[strum(serialize = "seeked")]
    OnSeeked,
    #[strum(serialize = "seeking")]
    OnSeeking,
    #[strum(serialize = "select")]
    OnSelect,
    #[strum(serialize = "selectstart")]
    OnSelectStart,
    #[strum(serialize = "show")]
    OnShow,
    #[strum(serialize = "stalled")]
    OnStalled,
    #[strum(serialize = "submit")]
    OnSubmit,
    #[strum(serialize = "suspend")]
    OnSuspend,
    #[strum(serialize = "timeupdate")]
    OnTimeUpdate,
    #[strum(serialize = "toggle")]
    OnToggle,
    #[strum(serialize = "touchcancel")]
    OnTouchCancel,
    #[strum(serialize = "touchend")]
    OnTouchEnd,
    #[strum(serialize = "touchmove")]
    OnTouchMove,
    #[strum(serialize = "touchstart")]
    OnTouchStart,
    #[strum(serialize = "transitioncancel")]
    OnTransitionCancel,
    #[strum(serialize = "transitionend")]
    OnTransitionEnd,
    #[strum(serialize = "transitionrun")]
    OnTransitionRun,
    #[strum(serialize = "transitionstart")]
    OnTransitionStart,
    #[strum(serialize = "volumechange")]
    OnVolumeChange,
    #[strum(serialize = "waiting")]
    OnWaiting,
    #[strum(serialize = "wheel")]
    OnWheel,
}
