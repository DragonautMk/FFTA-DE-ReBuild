package com.fftarecomp.port

import org.junit.Assert.*
import org.junit.Test

class FpsMeterTest {
    @Test fun countsEmulatedFramesSeparatelyFromDrawsAndResetsAfterPause() {
        val meter=FpsMeter(0,10)
        assertNull(meter.record(1,500_000_000,35))
        val rates=meter.record(3,1_000_000_000,60)!!
        assertEquals(4.0,rates.emulated,0.001)
        assertEquals(50.0,rates.drawn,0.001)
        meter.reset(5_000_000_000,100)
        val afterPause=meter.record(2,6_000_000_000,125)!!
        assertEquals(2.0,afterPause.emulated,0.001)
        assertEquals(25.0,afterPause.drawn,0.001)
    }
}
