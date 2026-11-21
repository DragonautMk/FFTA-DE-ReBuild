plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

val previewKeystore = providers.environmentVariable("FFTA_PREVIEW_KEYSTORE").orNull
val previewPasswordFile = providers.environmentVariable("FFTA_PREVIEW_PASSWORD_FILE").orNull
require((previewKeystore == null) == (previewPasswordFile == null)) {
    "Both preview signing files must be provided together"
}

android {
    namespace = "com.fftarecomp.port"
    compileSdk {
        version = release(37) { minorApiLevel = 2 }
    }
    buildToolsVersion = "37.0.0"
    ndkVersion = "30.0.16248370"

    defaultConfig {
        applicationId = "com.fftarecomp.port.preview"
        minSdk = 24
        targetSdk = 35
        versionCode = 33
        versionName = "1.2.0-android-preview.6"
        ndk { abiFilters += "arm64-v8a" }

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    if (previewKeystore != null && previewPasswordFile != null) {
        signingConfigs {
            create("persistentPreview") {
                val password = file(previewPasswordFile).readText().trimEnd('\r', '\n')
                require(password.length >= 6) { "Preview signing password is too short" }
                storeFile = file(previewKeystore)
                storePassword = password
                keyAlias = "ffta-preview"
                keyPassword = password
            }
        }
    }
    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        debug {
            isDebuggable = true
            if (previewKeystore != null) {
                signingConfig = signingConfigs.getByName("persistentPreview")
            }
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
    packaging {
        jniLibs {
            useLegacyPackaging = true
            if (providers.gradleProperty("fftaProfileSymbols").isPresent) {
                keepDebugSymbols += "**/libgba_android.so"
            }
        }
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.material)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)

    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
}
