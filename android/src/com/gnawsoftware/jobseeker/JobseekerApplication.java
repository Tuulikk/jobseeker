package com.gnawsoftware.jobseeker;

import android.app.Application;

public class JobseekerApplication extends Application {
    @Override
    public void onCreate() {
        super.onCreate();
        System.out.println("JobseekerApp: Application created");
        
        // Test network immediately on a background thread from Application context
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    System.out.println("JobseekerApp: Test fetch starting...");
                    String result = HttpFetcher.httpGet("https://google.com");
                    System.out.println("JobseekerApp: Test fetch result: " + (result != null ? "SUCCESS" : "FAIL"));
                } catch (Exception e) {
                    System.err.println("JobseekerApp: Test fetch crashed: " + e.getMessage());
                }
            }
        }).start();
    }
}
