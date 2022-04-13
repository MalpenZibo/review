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

/// The Effect Hook lets you perform side effects in components
///
/// # Examples
///
/// Without cleanup function and without dependencies
/// ```rust
/// # use review::{VNode, log, component, use_effect};
/// # use review::Tag::Div;
/// #[component(Example)]
/// pub fn example() -> VNode {
///     use_effect(
///         || {
///             log::info!("hello!");
///             None::<fn()>
///         },
///         None::<()>
///     );
///
///     Div.into()
/// }
/// ```
///
/// With cleanup function and without dependencies
/// ```rust
/// # use review::{VNode, component, use_effect};
/// # use review::Tag::Div;
/// #[component(Example)]
/// pub fn example() -> VNode {
///     use_effect(
///         || {
///             log::info!("hello!");
///             Some(|| log::info!("clean"))
///         },
///         None::<()>
///     );
///
///     Div.into()
/// }
/// ```
///
/// With cleanup function and dependencies
/// ```rust
/// # use review::{component, VNode, log, use_effect};
/// # use review::Tag::Div;
/// #[component(Example)]
/// pub fn example() -> VNode {
///     use_effect(
///         || {
///             log::info!("hello!");
///             Some(|| log::info!("clean"))
///         },
///         Some(()) // run only one time because () never change
///     );
///
///     Div.into()
/// }
/// ```
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn effect_call() {
        let mut context = HookContext::default();

        let counter = Rc::new(RefCell::new(0));

        let hook_context = &mut (0, &mut context);

        use_effect(
            {
                let counter = counter.clone();
                move || {
                    let mut c_mut = counter.borrow_mut();
                    *c_mut += 1;

                    None::<fn()>
                }
            },
            None::<()>,
        )
        .build(hook_context);

        for h in context.hooks.iter_mut() {
            h.post_render();
        }
        assert_eq!(*counter.borrow(), 1);
    }

    #[test]
    fn effect_call_with_dependency() {
        let mut context = HookContext::default();

        let mut dep = 0;

        let counter = Rc::new(RefCell::new(0));

        let effect = |hook_context: &mut (usize, &mut HookContext), dep: u32| {
            use_effect(
                {
                    let counter = counter.clone();
                    move || {
                        let mut c_mut = counter.borrow_mut();
                        *c_mut += 1;

                        None::<fn()>
                    }
                },
                Some(dep),
            )
            .build(hook_context);
        };

        {
            let hook_context = &mut (0, &mut context);
            (effect)(hook_context, dep);

            for h in context.hooks.iter_mut() {
                h.post_render();
            }
            assert_eq!(*counter.borrow(), 1);
        }

        context.counter = 0;

        {
            let hook_context = &mut (0, &mut context);
            (effect)(hook_context, dep);

            for h in context.hooks.iter_mut() {
                h.post_render();
            }
            assert_eq!(*counter.borrow(), 1);
        }

        dep += 1;
        context.counter = 0;

        {
            let hook_context = &mut (0, &mut context);
            (effect)(hook_context, dep);

            for h in context.hooks.iter_mut() {
                h.post_render();
            }
            assert_eq!(*counter.borrow(), 2);
        }
    }

    #[test]
    fn effect_call_with_cleanup() {
        let mut context = HookContext::default();

        let counter = Rc::new(RefCell::new(0));
        let clean = Rc::new(RefCell::new(1));

        let hook_context = &mut (0, &mut context);

        use_effect(
            {
                let counter = counter.clone();
                let clean = clean.clone();
                move || {
                    let mut c_mut = counter.borrow_mut();
                    *c_mut += 1;

                    let clean = clean.clone();
                    Some(move || {
                        let mut c_mut = clean.borrow_mut();
                        *c_mut = 0;
                    })
                }
            },
            None::<()>,
        )
        .build(hook_context);

        for h in context.hooks.iter_mut() {
            h.post_render();
        }
        assert_eq!(*counter.borrow(), 1);
        assert_eq!(*clean.borrow(), 0);
    }
}
