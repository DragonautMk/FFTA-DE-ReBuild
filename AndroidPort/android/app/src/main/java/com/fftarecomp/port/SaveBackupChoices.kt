package com.fftarecomp.port

data class SaveBackupChoice(val id:String,val valid:Boolean)

fun parseSaveBackupChoices(data:String):List<SaveBackupChoice> {
    require(!data.startsWith("ERROR|")) { data.substringAfter('|') }
    return data.lineSequence().filter{it.isNotEmpty()}.map { line ->
        val parts=line.split('|')
        require(parts.size==2 && parts[0].matches(Regex("[0-9]{20}-[0-9a-f]{64}")) && parts[1] in listOf("ready","damaged")) {
            "Invalid backup list"
        }
        SaveBackupChoice(parts[0],parts[1]=="ready")
    }.take(2).toList()
}
