package com.lingualore.desktop

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.telephony.SubscriptionManager
import android.telephony.TelephonyManager
import androidx.activity.enableEdgeToEdge
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

class MainActivity : TauriActivity() {
  private val phonePermissionRequestCode = 6201
  private val permissionRequested = "__PERMISSION_REQUESTED__"

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  fun getAndroidId(): String {
    return Settings.Secure.getString(contentResolver, Settings.Secure.ANDROID_ID) ?: ""
  }

  @Suppress("DEPRECATION")
  fun readPrimaryPhoneNumber(): String {
    val permissions = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      arrayOf(Manifest.permission.READ_PHONE_NUMBERS, Manifest.permission.READ_PHONE_STATE)
    } else {
      arrayOf(Manifest.permission.READ_PHONE_STATE)
    }
    val missing = permissions.filter {
      ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
    }.toTypedArray()
    if (missing.isNotEmpty()) {
      ActivityCompat.requestPermissions(this, missing, phonePermissionRequestCode)
      return permissionRequested
    }

    return try {
      val subscriptionManager = getSystemService(SubscriptionManager::class.java)
      val primarySubscription = subscriptionManager
        .activeSubscriptionInfoList
        ?.sortedBy { it.simSlotIndex }
        ?.firstOrNull()
      val subscriptionNumber = if (primarySubscription != null && Build.VERSION.SDK_INT >= 33) {
        subscriptionManager.getPhoneNumber(primarySubscription.subscriptionId)
      } else {
        primarySubscription?.number
      }
      if (!subscriptionNumber.isNullOrBlank()) {
        subscriptionNumber
      } else {
        val telephonyManager = getSystemService(Context.TELEPHONY_SERVICE) as TelephonyManager
        telephonyManager.line1Number ?: ""
      }
    } catch (_: SecurityException) {
      ""
    }
  }
}
