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
    compileSdk = 35
    ndkVersion = "28.2.13676358"

    defaultConfig {
        applicationId = "com.fftarecomp.port.preview"
        minSdk = 24
        targetSdk = 35
        versionCode = 33
        versionName = "1.2.0-android-preview.6"
        ndk { abiFilters += "arm64-v8a" }

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        multiDexEnabled = true
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

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_11)
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    implementation("androidx.activity:activity-compose:1.10.0")
    implementation(platform("androidx.compose:compose-bom:2025.02.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.6.1")
    androidTestImplementation(platform("androidx.compose:compose-bom:2025.02.00"))
    androidTestImplementation("androidx.compose.ui:ui-test-junit4")
}
