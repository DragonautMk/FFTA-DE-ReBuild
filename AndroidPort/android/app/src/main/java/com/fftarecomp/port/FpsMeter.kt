package com.fftarecomp.port

/** Counts emulated frames and GL draw callbacks, not display scanouts. */
data class FpsRates(val emulated: Double, val drawn: Double)

internal class FpsMeter(now: Long, drawn: Long) {
    private var start = now
    private var firstDrawn = drawn
    private var emulated = 0L

    fun reset(now: Long, drawn: Long) {
        start = now
        firstDrawn = drawn
        emulated = 0
    }

    fun record(frames: Int, now: Long, drawn: Long): FpsRates? {
        emulated += frames
        val elapsed = now - start
        if (elapsed < 1_000_000_000L) return null
        val rates = FpsRates(emulated * 1e9 / elapsed, (drawn - firstDrawn) * 1e9 / elapsed)
        reset(now, drawn)
        return rates
    }
}
