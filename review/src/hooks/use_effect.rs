use crate::fiber::FiberId;
use crate::Hook;
use crate::HookBuilder;
use crate::HookContext;
use std::any::Any;
use std::fmt::Debug;
use std::fmt::Formatter;

pub struct UseEffectBuilder<E, D: PartialEq> {
    effect: E,
    dependencies: Option<D>,
}

pub fn use_effect<E: Fn() -> Option<C> + 'static, C: Fn() + 'static, D: Any + PartialEq>(
    effect: E,
    dependencies: Option<D>,
) -> UseEffectBuilder<E, D> {
    UseEffectBuilder {
        effect,
        dependencies,
    }
}

struct EffectHook<E: Fn() -> Option<C>, C: Fn(), D: PartialEq> {
    effect: E,
    current_dependencies: Option<D>,
    last_dependencies: Option<D>,
}

impl<E: Fn() -> Option<C> + 'static, C: Fn() + 'static, D: Any + PartialEq> Debug
    for EffectHook<E, C, D>
{
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl<E: Fn() -> Option<C> + 'static, C: Fn() + 'static, D: Any + PartialEq> Hook
    for EffectHook<E, C, D>
{
    fn post_render(&mut self) {
        if self.current_dependencies.is_none()
            || self.current_dependencies != self.last_dependencies
        {
            let cleanup = (self.effect)();
            if let Some(cleanup) = cleanup {
                cleanup();
            }
            self.last_dependencies = self.current_dependencies.take();
        }
    }
}

impl<E: Fn() -> Option<C> + 'static, C: Fn() + 'static, D: Any + PartialEq> HookBuilder<()>
    for UseEffectBuilder<E, D>
{
    fn build(self, (_, hook_context): &mut (FiberId, &mut HookContext)) {
        let hook_position = hook_context.counter;
        hook_context.counter += 1;
        if hook_position >= hook_context.hooks.len() {
            let initial_value = EffectHook {
                effect: self.effect,
                current_dependencies: self.dependencies,
                last_dependencies: None,
            };
            hook_context.hooks.push(Box::new(initial_value));
        } else {
            let hook: &mut EffectHook<E, C, D> = hook_context.get_mut_hook(hook_position);

            hook.effect = self.effect;
            hook.current_dependencies = self.dependencies;
        }
    }
}
