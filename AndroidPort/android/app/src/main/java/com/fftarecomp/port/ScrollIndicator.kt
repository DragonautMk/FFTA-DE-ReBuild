package com.fftarecomp.port

import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

/** A persistent indicator appears whenever a vertical panel has hidden content. */
@Composable
fun Modifier.scrollWithIndicator():Modifier {
    val scroll=rememberScrollState()
    return this.drawWithContent {
        drawContent()
        val max=scroll.maxValue
        if(max>0 && max<Int.MAX_VALUE && size.height>0f) {
            val viewport=scroll.viewportSize.toFloat().coerceAtLeast(1f)
            val thumb=(size.height*viewport/(viewport+max)).coerceIn(24.dp.toPx().coerceAtMost(size.height),size.height)
            val top=(size.height-thumb)*scroll.value.toFloat()/max
            val width=4.dp.toPx()
            val left=(size.width-width).coerceAtLeast(0f)
            drawRoundRect(Color(0x55765A25),Offset(left,0f),Size(width,size.height),CornerRadius(width/2))
            drawRoundRect(Color(0xFF765A25),Offset(left,top),Size(width,thumb),CornerRadius(width/2))
        }
    }.verticalScroll(scroll).padding(end=12.dp)
}
