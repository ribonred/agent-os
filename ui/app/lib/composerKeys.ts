// When the owner is writing, Enter is send and Shift+Enter is a new
// line -- the same two keys every other quiet chat box uses. A
// candidate-window Enter (East Asian input) is not a send: treating it
// as one would fire the half-composed syllable.

export type ComposerKeyEvent = {
  key: string;
  shiftKey: boolean;
  isComposing: boolean;
  keyCode: number;
};

/// True when this key should send the draft. Callers still decide
/// whether the draft is empty or the field is disabled.
export function shouldSubmitComposer(event: ComposerKeyEvent): boolean {
  if (event.key !== "Enter" || event.shiftKey) return false;
  // 229 is the keyCode browsers report while an IME is composing.
  if (event.isComposing || event.keyCode === 229) return false;
  return true;
}
