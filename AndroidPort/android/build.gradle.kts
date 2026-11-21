plugins {
    alias(libs.plugins.android.application) apply false
    // Resolve KGP at the Compose compiler version for AGP built-in Kotlin.
    alias(libs.plugins.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
}
