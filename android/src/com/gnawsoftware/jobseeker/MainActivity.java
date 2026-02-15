package com.gnawsoftware.jobseeker;

import android.app.NativeActivity;
import android.os.Bundle;
import android.util.Log;
import java.net.InetAddress;

/**
 * MainActivity extends NativeActivity to provide a Java entry point.
 * This ensures the process is correctly initialized as a standard Android app,
 * which helps WayDroid grant the necessary network permissions (inet group).
 */
public class MainActivity extends NativeActivity {
    private static final String TAG = "JobseekerMainActivity";

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        Log.i(TAG, "Starting MainActivity...");
        
        // This dummy call on the main thread helps "wake up" the network stack
        // and ensures the app is associated with the 'inet' group.
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    InetAddress.getByName("8.8.8.8");
                    Log.i(TAG, "Network stack poked successfully");
                } catch (Exception e) {
                    Log.w(TAG, "Initial network poke failed: " + e.getMessage());
                }
            }
        }).start();

        super.onCreate(savedInstanceState);
    }
}
