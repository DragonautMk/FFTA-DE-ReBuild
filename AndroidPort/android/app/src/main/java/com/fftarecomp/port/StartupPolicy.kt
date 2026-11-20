package com.fftarecomp.port

enum class StartupRoute { SETUP, LOADING }
object StartupPolicy {
    fun route(ready:Boolean,romPresent:Boolean,nativeError:Boolean,recorded:Boolean=true,benchmark:Boolean=false):StartupRoute {
        // Normal launches always expose the ROM and save-import menu. Debug
        // benchmarks retain their existing unattended startup path.
        if(!benchmark)return StartupRoute.SETUP
        if(nativeError || !romPresent)return StartupRoute.SETUP
        return if(ready || !recorded)StartupRoute.LOADING else StartupRoute.SETUP
    }
}
