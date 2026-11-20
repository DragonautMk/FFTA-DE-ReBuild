package com.fftarecomp.port

import java.io.File
import java.nio.ByteBuffer
import java.nio.channels.FileChannel
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.StandardOpenOption
import java.security.MessageDigest

/** Paths are relative to either app-private storage or a user-selected ROM folder. */
internal data class SaveLayout(val root:File,val save:File,val slots:List<File>,val backups:File,val imports:File) {
    companion object {
        fun privateFiles(root:File,romHash:String):SaveLayout {
            val states=File(root,"SaveStates/$romHash")
            return SaveLayout(root,File(root,"ffta.sav"),(1..9).map{File(states,"slot-$it.ffstate")},
                File(root,"SaveBackups/$romHash/ffta.sav"),File(states,"ImportBackups"))
        }
        fun romFolder(root:File,romName:String,romHash:String):SaveLayout {
            require(romName.isNotBlank() && romName==File(romName).name && romName!="." && romName!="..") {"Invalid ROM filename"}
            val base=romName.substringBeforeLast('.',romName).ifBlank {romName}
            val saveName="$base.sav"
            require(!saveName.equals(romName,ignoreCase=true)){"Rename the ROM so its filename is different from its save file"}
            return SaveLayout(root,File(root,saveName),(1..9).map{File(root,"$base.slot-$it.ffstate")},
                File(root,"SaveBackups/$romHash/$saveName"),File(root,"SaveStateImportBackups/$romHash"))
        }
    }
}

internal class SaveFolderConflict(val names:List<String>):IllegalStateException("Different saves already exist in the selected folder")

internal object RomFolderFiles {
    private const val MAX_STATE=65L*1024*1024
    private val backupName=Regex("backup-[0-9]{20}-[0-9a-f]{64}\\.sav")
    private val importName=Regex("slot-[1-9]-[0-9a-f]{64}\\.ffstate")
    private data class Copy(val source:File,val target:File,val digest:String,val length:Long)

    fun hash(file:File):String {
        val digest=MessageDigest.getInstance("SHA-256")
        file.inputStream().buffered().use { input ->
            val buffer=ByteArray(65536)
            while(true){val count=input.read(buffer);if(count<0)break;digest.update(buffer,0,count)}
        }
        return digest.digest().joinToString(""){"%02x".format(it.toInt() and 255)}
    }

    fun findRom(folder:File,rom:File):File {
        require(folder.isDirectory){"The selected folder is unavailable"}
        require(rom.isFile){"Select a ROM first"}
        val expected=hash(rom);val size=rom.length()
        val matches=checkNotNull(folder.listFiles()){ "Cannot read the selected folder" }.filter {
            it.isFile && !Files.isSymbolicLink(it.toPath()) && it.length()==size && hash(it)==expected
        }
        require(matches.size==1){if(matches.isEmpty())"The selected folder does not contain the selected ROM" else "The folder contains multiple copies of the selected ROM"}
        return matches.single()
    }

    private fun inside(root:File,file:File) {
        val base=root.canonicalFile.toPath();val path=file.canonicalFile.toPath()
        require(path.startsWith(base) && path!=base){"A save path leaves its selected folder"}
        var current=file.toPath()
        val rootPath=root.absoluteFile.toPath().normalize()
        while(current.startsWith(rootPath) && current!=rootPath) {
            require(!Files.isSymbolicLink(current)){"A save path is a symbolic link"}
            current=current.parent ?: break
        }
    }

    /** Preflight every destination before creating anything; never replace an existing file. */
    fun copySaves(source:SaveLayout,destination:SaveLayout,keepExisting:Boolean=false,publish:(File,File)->Unit):Int {
        val pairs=mutableListOf<Pair<File,File>>()
        pairs+=source.save to destination.save
        source.slots.zip(destination.slots).forEach{pairs+=it}
        fun addDirectory(from:File,to:File,allowed:Regex) {
            if(!from.exists())return
            require(from.isDirectory && !Files.isSymbolicLink(from.toPath())){"A save-backup directory is invalid"}
            checkNotNull(from.listFiles()){ "Cannot list save backups" }.filter{allowed.matches(it.name)}
                .sortedBy{it.name}.forEach{pairs+=it to File(to,it.name)}
        }
        addDirectory(source.backups,destination.backups,backupName)
        addDirectory(source.imports,destination.imports,importName)
        val pending=mutableListOf<Copy>()
        val conflicts=mutableListOf<String>()
        for((from,to) in pairs) {
            inside(source.root,from);inside(destination.root,to)
            if(!from.exists())continue
            require(Files.isRegularFile(from.toPath(),LinkOption.NOFOLLOW_LINKS)){"A source save is not a regular file"}
            val size=from.length();require(size in 1..MAX_STATE){"A save has an invalid size"}
            val digest=hash(from)
            if(to.exists() || Files.isSymbolicLink(to.toPath())) {
                require(Files.isRegularFile(to.toPath(),LinkOption.NOFOLLOW_LINKS)){"A destination save is not a regular file"}
                if(to.length()!=size || hash(to)!=digest)conflicts+=to.name
            } else pending+=Copy(from,to,digest,size)
        }
        if(conflicts.isNotEmpty() && !keepExisting)throw SaveFolderConflict(conflicts)
        for(copy in pending) {
            inside(destination.root,copy.target)
            val parent=checkNotNull(copy.target.parentFile)
            check(parent.mkdirs() || parent.isDirectory){"Cannot create the save directory"}
            inside(destination.root,copy.target)
            val temporary=Files.createTempFile(parent.toPath(),".ffta-migrate-",".tmp").toFile()
            try {
                FileChannel.open(temporary.toPath(),StandardOpenOption.WRITE,LinkOption.NOFOLLOW_LINKS).use { out ->
                    copy.source.inputStream().buffered().use { input ->
                        val bytes=ByteArray(65536)
                        while(true){val count=input.read(bytes);if(count<0)break
                            val block=ByteBuffer.wrap(bytes,0,count);while(block.hasRemaining())out.write(block)}
                    }
                    out.force(true)
                }
                check(copy.source.length()==copy.length && hash(copy.source)==copy.digest &&
                    temporary.length()==copy.length && hash(temporary)==copy.digest){"A save changed during copying; originals were kept"}
                publish(temporary,copy.target)
                check(copy.target.length()==copy.length && hash(copy.target)==copy.digest){"A copied save failed verification; originals were kept"}
            } finally {Files.deleteIfExists(temporary.toPath())}
        }
        return pending.size
    }
}
