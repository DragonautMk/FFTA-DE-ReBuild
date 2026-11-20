package com.fftarecomp.port

import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File
import java.nio.file.Files

class RomFolderFilesTest {
    @get:Rule val temporary=TemporaryFolder()
    private fun put(file:File,text:String){file.parentFile!!.mkdirs();file.writeText(text)}
    private fun copy(source:SaveLayout,dest:SaveLayout,keepExisting:Boolean=false)=
        RomFolderFiles.copySaves(source,dest,keepExisting){temp,target->Files.move(temp.toPath(),target.toPath());Unit}

    @Test fun migratesSavesBesideRomWithoutRemovingOriginals() {
        val privateRoot=temporary.newFolder("private");val folder=temporary.newFolder("roms")
        val source=SaveLayout.privateFiles(privateRoot,"rom-hash");val dest=SaveLayout.romFolder(folder,"My Game.gba","rom-hash")
        val backup="backup-00000000000000000001-"+"a".repeat(64)+".sav"
        val history="slot-1-"+"b".repeat(64)+".ffstate"
        put(source.save,"save");put(source.slots[0],"state");put(File(source.backups,backup),"backup");put(File(source.imports,history),"history")
        assertEquals(4,copy(source,dest))
        assertEquals("save",File(folder,"My Game.sav").readText())
        assertEquals("state",File(folder,"My Game.slot-1.ffstate").readText())
        assertEquals("backup",File(dest.backups,backup).readText())
        assertEquals("history",File(dest.imports,history).readText())
        assertEquals("save",source.save.readText());assertEquals("state",source.slots[0].readText())
        assertEquals(0,copy(source,dest))
    }
    @Test fun detectsConflictsBeforeCopyingAnySave() {
        val source=SaveLayout.privateFiles(temporary.newFolder("private"),"rom-hash")
        val dest=SaveLayout.romFolder(temporary.newFolder("roms"),"game.gba","rom-hash")
        put(source.save,"current");put(source.slots[0],"old state");put(dest.slots[0],"other state")
        val error=runCatching{copy(source,dest)}.exceptionOrNull()
        assertTrue(error is SaveFolderConflict);assertFalse(dest.save.exists());assertEquals("other state",dest.slots[0].readText())
        assertEquals("current",source.save.readText())
        assertEquals(1,copy(source,dest,keepExisting=true))
        assertEquals("other state",dest.slots[0].readText());assertEquals("current",dest.save.readText())
    }
    @Test fun detectsLinksThatEscapeTheChosenFolder() {
        val source=SaveLayout.privateFiles(temporary.newFolder("private"),"rom-hash")
        val folder=temporary.newFolder("roms");val dest=SaveLayout.romFolder(folder,"game.gba","rom-hash")
        put(source.save,"current");val outside=temporary.newFile("outside.sav");outside.writeText("outside")
        Files.createSymbolicLink(dest.save.toPath(),outside.toPath())
        assertTrue(runCatching{copy(source,dest)}.isFailure)
        assertEquals("outside",outside.readText());assertEquals("current",source.save.readText())
    }
    @Test fun matchesRomByContentAndRejectsAmbiguity() {
        val folder=temporary.newFolder("roms");val rom=temporary.newFile("private.gba");rom.writeText("synthetic rom")
        val match=File(folder,"game.gba");match.writeText("synthetic rom")
        File(folder,"other.gba").writeText("different rom")
        assertEquals(match,RomFolderFiles.findRom(folder,rom))
        File(folder,"duplicate.gba").writeText("synthetic rom")
        assertTrue(runCatching{RomFolderFiles.findRom(folder,rom)}.isFailure)
    }
    @Test fun migratesBetweenRomFoldersWithDifferentNames() {
        val source=SaveLayout.romFolder(temporary.newFolder("one"),"A.gba","hash")
        val dest=SaveLayout.romFolder(temporary.newFolder("two"),"B.gba","hash")
        put(source.save,"progress");put(source.slots[2],"slot three")
        assertEquals(2,copy(source,dest))
        assertEquals("progress",File(dest.root,"B.sav").readText())
        assertEquals("slot three",File(dest.root,"B.slot-3.ffstate").readText())
    }
    @Test fun concurrentDestinationIsNeverOverwritten() {
        val source=SaveLayout.privateFiles(temporary.newFolder("private"),"hash")
        val dest=SaveLayout.romFolder(temporary.newFolder("roms"),"game.gba","hash")
        put(source.save,"source")
        val result=runCatching {
            RomFolderFiles.copySaves(source,dest){temp,target ->
                target.writeText("concurrent");Files.move(temp.toPath(),target.toPath());Unit
            }
        }
        assertTrue(result.isFailure);assertEquals("concurrent",dest.save.readText());assertEquals("source",source.save.readText())
    }
    @Test fun romNamedLikeItsSaveIsRejected() {
        val folder=temporary.newFolder("roms");val rom=File(folder,"game.sav");rom.writeText("rom")
        assertTrue(runCatching{SaveLayout.romFolder(folder,rom.name,"hash")}.isFailure)
        assertEquals("rom",rom.readText())
    }
}
