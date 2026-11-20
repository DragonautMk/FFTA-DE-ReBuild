package com.fftarecomp.port

import android.content.Context
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.os.storage.StorageManager
import android.provider.DocumentsContract
import java.io.File

/** The folder grant identifies the location; the optional all-files grant permits native file I/O. */
internal class RomFolderLocation(private val context:Context) {
    private val prefs=context.getSharedPreferences("rom_save_folder",Context.MODE_PRIVATE)
    val selected:Boolean get()=prefs.contains("tree")

    fun privateLayout(rom:File)=SaveLayout.privateFiles(context.filesDir,RomFolderFiles.hash(rom))

    fun layout(rom:File):SaveLayout {
        val saved=prefs.getString("tree",null) ?: return privateLayout(rom)
        val folder=directory(Uri.parse(saved))
        val found=RomFolderFiles.findRom(folder,rom)
        val hash=RomFolderFiles.hash(rom)
        if(prefs.getString("rom_hash",null)==hash) {
            check(prefs.getString("rom_name",null)==found.name) {
                "The ROM filename changed. Select its folder again to copy the existing saves safely."
            }
        }
        return SaveLayout.romFolder(folder,found.name,hash)
    }

    /** Retain the old filenames as the migration source if the ROM was renamed. */
    fun sourceLayout(rom:File):SaveLayout {
        val saved=prefs.getString("tree",null) ?: return privateLayout(rom)
        val hash=RomFolderFiles.hash(rom)
        val previousName=prefs.getString("rom_name",null)
        if(prefs.getString("rom_hash",null)==hash && previousName!=null) {
            return SaveLayout.romFolder(directory(Uri.parse(saved)),previousName,hash)
        }
        return layout(rom)
    }

    fun select(uri:Uri,rom:File,source:SaveLayout,keepExisting:Boolean=false):Pair<SaveLayout,Int> {
        val folder=directory(uri)
        val found=RomFolderFiles.findRom(folder,rom)
        val hash=RomFolderFiles.hash(rom)
        val destination=SaveLayout.romFolder(folder,found.name,hash)
        val copied=RomFolderFiles.copySaves(source,destination,keepExisting) {temp,target ->
            check(NativeBridge.gbaPublishSaveCopy(temp.absolutePath,target.absolutePath)==1){NativeBridge.gbaStatus()}
        }
        check(prefs.edit().putString("tree",uri.toString()).putString("rom_hash",hash)
            .putString("rom_name",found.name).commit()){ "Cannot remember the save folder" }
        return destination to copied
    }

    fun usePrivate():Boolean=prefs.edit().remove("tree").remove("rom_hash").remove("rom_name").commit()

    private fun directory(uri:Uri):File {
        check(Build.VERSION.SDK_INT>=30){"ROM-folder saves require Android 11 or newer"}
        check(Environment.isExternalStorageManager()){"Allow access to all files to use the ROM folder"}
        require(uri.authority=="com.android.externalstorage.documents" && DocumentsContract.isTreeUri(uri)) {
            "Choose a local device folder, not a cloud or virtual folder"
        }
        val id=DocumentsContract.getTreeDocumentId(uri);val parts=id.split(':',limit=2)
        require(parts.size==2 && parts[1].isNotBlank()){ "Choose a subfolder containing your ROM" }
        val segments=parts[1].split('/')
        require(segments.all{it.isNotEmpty() && it!="." && it!=".."}){"Invalid folder path"}
        val manager=context.getSystemService(StorageManager::class.java)
        val volume=manager.storageVolumes.firstOrNull {
            if(parts[0].equals("primary",ignoreCase=true))it.isPrimary else it.uuid.equals(parts[0],ignoreCase=true)
        } ?: error("The selected storage device is unavailable")
        val root=checkNotNull(volume.directory){"Cannot access the selected storage device"}.canonicalFile
        val folder=File(root,segments.joinToString("/")).canonicalFile
        require(folder.toPath().startsWith(root.toPath()) && folder!=root && folder.isDirectory) {
            "The selected folder is unavailable"
        }
        return folder
    }
}
