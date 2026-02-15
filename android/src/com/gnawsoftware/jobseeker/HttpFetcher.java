package com.gnawsoftware.jobseeker;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.concurrent.Callable;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;

/**
 * HTTP fetcher using Android's HttpURLConnection.
 * Uses a separate Java thread to ensure network permissions are correctly handled.
 */
public class HttpFetcher {

    private static final String TAG = "HttpFetcher";

    static {
        try {
            System.loadLibrary("Jobseeker");
            System.out.println(TAG + ": Native library loaded successfully");
        } catch (UnsatisfiedLinkError e) {
            System.err.println(TAG + ": Failed to load native library: " + e.getMessage());
        }
    }

    public static String httpGet(final String url) {
        System.out.println(TAG + ": httpGet called for: " + url);
        
        FutureTask<String> task = new FutureTask<>(new Callable<String>() {
            @Override
            public String call() throws Exception {
                return performGet(url);
            }
        });

        new Thread(task).start();

        try {
            return task.get(30, TimeUnit.SECONDS);
        } catch (Exception e) {
            System.err.println(TAG + ": Task execution failed: " + e.getMessage());
            return null;
        }
    }

    private static String performGet(String url) {
        HttpURLConnection connection = null;
        try {
            URL urlObj = new URL(url);
            connection = (HttpURLConnection) urlObj.openConnection();
            connection.setRequestMethod("GET");
            connection.setConnectTimeout(10000);
            connection.setReadTimeout(20000);
            connection.setRequestProperty("Accept", "application/json");

            int responseCode = connection.getResponseCode();
            if (responseCode == HttpURLConnection.HTTP_OK) {
                BufferedReader reader = new BufferedReader(
                    new InputStreamReader(connection.getInputStream())
                );
                StringBuilder response = new StringBuilder();
                String line;
                while ((line = reader.readLine()) != null) {
                    response.append(line);
                }
                reader.close();
                System.out.println(TAG + ": Request successful, length: " + response.length());
                return response.toString();
            } else {
                System.err.println(TAG + ": HTTP error: " + responseCode);
                return null;
            }
        } catch (Exception e) {
            System.err.println(TAG + ": Network operation failed: " + e.getMessage());
            return null;
        } finally {
            if (connection != null) {
                connection.disconnect();
            }
        }
    }
}
