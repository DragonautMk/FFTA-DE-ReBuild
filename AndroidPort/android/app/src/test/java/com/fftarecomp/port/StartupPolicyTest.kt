package com.fftarecomp.port
import org.junit.Assert.assertEquals
import org.junit.Test
class StartupPolicyTest {
    @Test fun firstInstallShowsSetup(){assertEquals(StartupRoute.SETUP,StartupPolicy.route(false,false,false))}
    @Test fun preparedInstallShowsSetup(){assertEquals(StartupRoute.SETUP,StartupPolicy.route(true,true,false))}
    @Test fun olderInstallShowsSetup(){assertEquals(StartupRoute.SETUP,StartupPolicy.route(false,true,false,false))}
    @Test fun missingRomRecoversToSetup(){assertEquals(StartupRoute.SETUP,StartupPolicy.route(true,false,false))}
    @Test fun failedStoredRomRecoversToSetup(){assertEquals(StartupRoute.SETUP,StartupPolicy.route(false,true,false,true))}
    @Test fun nativeFailureRecoversToSetup(){assertEquals(StartupRoute.SETUP,StartupPolicy.route(true,true,true))}
    @Test fun benchmarkCanStillStartUnattended(){assertEquals(StartupRoute.LOADING,StartupPolicy.route(true,true,false,benchmark=true))}
    @Test fun benchmarkWithoutRomShowsSetup(){assertEquals(StartupRoute.SETUP,StartupPolicy.route(true,false,false,benchmark=true))}
}
