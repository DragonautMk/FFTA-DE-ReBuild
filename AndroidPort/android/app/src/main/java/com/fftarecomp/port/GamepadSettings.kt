package com.fftarecomp.port

import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.Window
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider

private val PadInk=Color(0xFF302519)
private val PadGold=Color(0xFFB68B35)
private val PadPaper=Color(0xFFF6EBD0)

@Composable
fun GamepadSettings(settings:GameSettings){
    val revision=settings.bindingRevision
    Text(if(settings.controllerConnected)"Controller connected" else "No controller connected",color=PadGold)
    Text("Select an action, then press the button you want to use on your gamepad.",fontSize=13.sp)
    PadAction.entries.forEach{action->
        val bound=remember(revision,action){settings.pad.bindings.getValue(action)}
        OutlinedButton(onClick={settings.captureAction=action},
            modifier=Modifier.fillMaxWidth(),shape=RoundedCornerShape(10.dp),
            contentPadding=PaddingValues(horizontal=12.dp,vertical=6.dp)){
            Row(Modifier.fillMaxWidth(),verticalAlignment=Alignment.CenterVertically,
                horizontalArrangement=Arrangement.SpaceBetween){
                Text(action.label,Modifier.weight(1f),fontFamily=FontFamily.Serif,fontSize=16.sp,color=PadInk)
                PadBadge(bound)
            }
        }
    }
    Row(verticalAlignment=Alignment.CenterVertically){
        Switch(settings.pad.leftStickMovement,{
            settings.pad.leftStickMovement=it;settings.bindingRevision++;settings.savePad()
        })
        Text("Left stick also moves the cursor",Modifier.weight(1f),fontSize=13.sp)
    }
    OutlinedButton(onClick={settings.pad.defaults();settings.bindingRevision++;settings.savePad()},
        modifier=Modifier.fillMaxWidth()){Text("Restore default layout")}
    Text("LT / RT: L / R. LB / RB: zoom out / in. L3: hold fast forward. R3: toggle. Guide: Pause (when available).",fontSize=12.sp)
    Text("Some Android controllers reserve Guide for the system; Pause can be assigned to another button.",fontSize=12.sp)
    settings.captureAction?.let{action->
        Dialog(onDismissRequest={settings.captureAction=null},properties=DialogProperties(usePlatformDefaultWidth=false)){
            CaptureControllerEvents(settings)
            Column(Modifier.widthIn(max=500.dp).fillMaxWidth(0.92f).heightIn(max=320.dp)
                .background(PadPaper,RoundedCornerShape(18.dp)).border(1.dp,PadGold,RoundedCornerShape(18.dp))
                .padding(18.dp).scrollWithIndicator(),verticalArrangement=Arrangement.spacedBy(10.dp)){
                Text("Assign "+action.label,fontFamily=FontFamily.Serif,fontSize=23.sp,color=PadInk)
                Text("Press the button you want to use on your gamepad.",fontSize=13.sp)
                Text("Waiting for controller input...",color=PadGold,fontSize=15.sp)
                Text("Triggers, D-pad and stick directions are also supported.",fontSize=12.sp)
                Text("An occupied button swaps with the previous binding.",fontSize=12.sp)
                OutlinedButton(onClick={settings.captureAction=null},modifier=Modifier.fillMaxWidth()){Text("Cancel")}
            }
        }
    }
}

/** Dialogs have their own Window, so Activity.dispatchKeyEvent alone cannot capture here. */
@Composable
private fun CaptureControllerEvents(settings:GameSettings){
    val view=LocalView.current
    DisposableEffect(view,settings){
        val window=generateSequence(view as android.view.View?){it.parent as? android.view.View}
            .filterIsInstance<DialogWindowProvider>().firstOrNull()?.window
        val original=window?.callback
        val callback=if(original!=null)object:Window.Callback by original {
            override fun dispatchKeyEvent(event:KeyEvent):Boolean {
                val controller=event.isFromSource(InputDevice.SOURCE_GAMEPAD) ||
                    event.device?.let{it.sources and InputDevice.SOURCE_GAMEPAD==InputDevice.SOURCE_GAMEPAD}==true
                if(controller && settings.captureKey(event))return true
                return original.dispatchKeyEvent(event)
            }
            override fun dispatchGenericMotionEvent(event:MotionEvent):Boolean {
                if(event.isFromSource(InputDevice.SOURCE_JOYSTICK) &&
                    event.action==MotionEvent.ACTION_MOVE && settings.captureMotion(event))return true
                return original.dispatchGenericMotionEvent(event)
            }
        }else null
        if(callback!=null)window?.callback=callback
        onDispose{if(window?.callback===callback && original!=null)window.callback=original}
    }
}

/** Crisp vector badges: colored face buttons, shoulders, triggers, d-pad and sticks. */
@Composable
private fun PadBadge(button:PadControl){
    val face=when(button){
        PadControl.A->Color(0xFF4CAF50);PadControl.B->Color(0xFFE0453A)
        PadControl.X->Color(0xFF2F6FD0);PadControl.Y->Color(0xFFF2C12E)
        else->null
    }
    val name=button.name
    val direction=when {
        name.endsWith("UP")->"\u2191"
        name.endsWith("DOWN")->"\u2193"
        name.endsWith("LEFT")->"\u2190"
        name.endsWith("RIGHT")->"\u2192"
        else->""
    }
    Row(Modifier.width(118.dp).heightIn(min=36.dp).semantics{contentDescription=button.label},
        verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(6.dp)){
        if(face!=null){
            Box(Modifier.size(32.dp).background(face,CircleShape).border(1.dp,PadInk,CircleShape),
                contentAlignment=Alignment.Center){
                Text(button.label,fontSize=19.sp,fontWeight=FontWeight.Bold,
                    color=if(button==PadControl.Y)PadInk else Color.White)
            }
        }else if(button in listOf(PadControl.UP,PadControl.DOWN,PadControl.LEFT,PadControl.RIGHT)){
            Box(Modifier.size(32.dp),contentAlignment=Alignment.Center){
                Canvas(Modifier.fillMaxSize()){
                    val w=size.width;val h=size.height
                    drawRoundRect(PadInk,Offset(w/3,0f),Size(w/3,h),androidx.compose.ui.geometry.CornerRadius(3f))
                    drawRoundRect(PadInk,Offset(0f,h/3),Size(w,h/3),androidx.compose.ui.geometry.CornerRadius(3f))
                }
                Text(direction,color=PadPaper,fontSize=18.sp,fontWeight=FontWeight.Bold)
            }
        }else{
            val stick=name.startsWith("LS")||name.startsWith("RS")
            val label=when {
                stick->if(name.startsWith("LS"))"LS" else "RS"
                button==PadControl.START->"\u25b6"
                button==PadControl.BACK->"\u25c0"
                button==PadControl.GUIDE->"G"
                else->button.label
            }
            Box(Modifier.size(34.dp).background(PadInk,if(stick)CircleShape else RoundedCornerShape(7.dp))
                .border(1.dp,PadGold,if(stick)CircleShape else RoundedCornerShape(7.dp)),contentAlignment=Alignment.Center){
                Text(if(stick&&direction.isNotEmpty())direction else label,color=PadPaper,fontSize=14.sp,fontWeight=FontWeight.Bold)
            }
        }
        if(face==null)Text(button.label,fontSize=11.sp,color=PadInk,lineHeight=12.sp)
    }
}
