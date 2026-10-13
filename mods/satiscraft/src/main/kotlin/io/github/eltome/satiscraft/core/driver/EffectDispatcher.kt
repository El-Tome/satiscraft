package io.github.eltome.satiscraft.core.driver

import io.github.eltome.satiscraft.core.contract.Effect
import org.slf4j.Logger
import kotlin.reflect.KClass

/**
 * Hands each effect of a tick to the one handler registered for its type. An effect without
 * a handler is logged once per type and dropped; a failing handler is logged. Neither stops
 * the effects that follow.
 */
class EffectDispatcher(private val logger: Logger) {
    private val handlers = HashMap<KClass<out Effect>, (Effect) -> Unit>()
    private val unhandled = HashSet<KClass<out Effect>>()

    /** Makes [handler] the handler of the effects of [type], which takes only one. */
    fun <E : Effect> register(type: KClass<E>, handler: (E) -> Unit) {
        check(type !in handlers) { "factory-core effect ${type.simpleName} already has a handler" }
        handlers[type] = { effect -> handler(type.java.cast(effect)) }
    }

    /** [register] for the effect type [E]. */
    inline fun <reified E : Effect> register(noinline handler: (E) -> Unit) = register(E::class, handler)

    /** Hands [effects] to their handlers, in order. */
    fun dispatch(effects: List<Effect>) = effects.forEach(::dispatch)

    private fun dispatch(effect: Effect) {
        val handler = handlers[effect::class]
        if (handler == null) {
            if (unhandled.add(effect::class)) {
                logger.warn("No handler for factory-core effect {}: dropped, and logged once per type", effect)
            }
            return
        }
        try {
            handler(effect)
        } catch (e: Exception) {
            logger.error("The handler of factory-core effect {} failed", effect, e)
        }
    }
}
