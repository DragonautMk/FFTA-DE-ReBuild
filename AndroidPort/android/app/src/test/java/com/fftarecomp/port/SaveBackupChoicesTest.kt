package com.fftarecomp.port

import org.junit.Assert.*
import org.junit.Test

class SaveBackupChoicesTest {
    @Test fun parsesEmptyReadyAndDamagedChoices() {
        val id="00000000000000000001-"+"a".repeat(64)
        assertTrue(parseSaveBackupChoices("").isEmpty())
        assertEquals(listOf(SaveBackupChoice(id,true),SaveBackupChoice(id,false)),
            parseSaveBackupChoices("$id|ready\n$id|damaged\n"))
    }
    @Test fun rejectsErrorsAndInvalidIdentifiers() {
        assertThrows(IllegalArgumentException::class.java){parseSaveBackupChoices("ERROR|Unavailable")}
        assertThrows(IllegalArgumentException::class.java){parseSaveBackupChoices("../file|ready")}
    }
}
