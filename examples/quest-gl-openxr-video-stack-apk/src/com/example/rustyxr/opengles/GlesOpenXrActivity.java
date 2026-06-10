package com.example.rustyxr.opengles;

import android.content.Intent;
import android.media.projection.MediaProjectionManager;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;

public final class GlesOpenXrActivity extends android.app.NativeActivity {
    private static final String TAG = "RustyXrGlesActivity";
    private static final int MEDIA_PROJECTION_REQUEST = 8711;
    private static final long DEFAULT_MEDIA_PROJECTION_DELAY_MS = 1600L;

    private MediaProjectionManager mediaProjectionManager;
    private final Handler mainHandler = new Handler(Looper.getMainLooper());

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        applyDefaultPassthroughLensIntentExtras();
        super.onCreate(savedInstanceState);
        mediaProjectionManager = (MediaProjectionManager) getSystemService(MEDIA_PROJECTION_SERVICE);
        requestMediaProjectionIfEnabled();
    }

    @Override
    protected void onNewIntent(Intent intent) {
        applyDefaultPassthroughLensIntentExtras(intent);
        super.onNewIntent(intent);
        setIntent(intent);
        requestMediaProjectionIfEnabled();
    }

    private void applyDefaultPassthroughLensIntentExtras() {
        Intent intent = getIntent();
        if (intent == null) {
            intent = new Intent();
            setIntent(intent);
        }
        applyDefaultPassthroughLensIntentExtras(intent);
    }

    private void applyDefaultPassthroughLensIntentExtras(Intent intent) {
        if (intent == null || intent.hasExtra("rustyxr.example") || intent.hasExtra("rustyxr.videoSource")) {
            return;
        }
        putDefaultStringExtra(intent, "rustyxr.videoSource", "direct-camera2-oes");
        putDefaultStringExtra(intent, "rustyxr.directCamera2OesLeftCameraId", "50");
        putDefaultStringExtra(intent, "rustyxr.directCamera2OesRightCameraId", "51");
        putDefaultStringExtra(intent, "rustyxr.directCamera2OesWidth", "1280");
        putDefaultStringExtra(intent, "rustyxr.directCamera2OesHeight", "1280");
        putDefaultStringExtra(intent, "rustyxr.directCamera2OesFrameRateHz", "50");
        putDefaultStringExtra(intent, "rustyxr.directCamera2OesProjectionGeometryProfile", "camera-projection");
        putDefaultStringExtra(intent, "rustyxr.directCamera2OesSourceSamplingMode", "screen-to-camera-homography");
        putDefaultStringExtra(intent, "rustyxr.cameraProjectionMode", "display-screen-homography");
        putDefaultStringExtra(intent, "rustyxr.processingLayer", "seam-blend");
        putDefaultStringExtra(intent, "rustyxr.projectionDepthMeters", "0.42");
        putDefaultStringExtra(intent, "rustyxr.cameraPreviewFovYDegrees", "68.0");
        putDefaultStringExtra(intent, "rustyxr.cameraPreviewOffsetYMeters", "0.0");
        putDefaultStringExtra(intent, "rustyxr.cameraRawOverlayOverscan", "1.0");
        putDefaultStringExtra(intent, "rustyxr.projectionAreaOffsetXUv", "0.0");
        putDefaultStringExtra(intent, "rustyxr.projectionAreaOffsetYUv", "-0.07");
        putDefaultStringExtra(intent, "rustyxr.cameraUvOffsetX", "0.0");
        putDefaultStringExtra(intent, "rustyxr.cameraUvOffsetY", "0.0");
        putDefaultStringExtra(intent, "rustyxr.cameraUvScaleX", "1.0");
        putDefaultStringExtra(intent, "rustyxr.cameraUvScaleY", "1.0");
        putDefaultStringExtra(intent, "rustyxr.cameraUvRadialK1", "0.0");
        putDefaultStringExtra(intent, "rustyxr.cameraUvRadialK2", "0.0");
        putDefaultStringExtra(intent, "rustyxr.projectionAreaOpacity", "0.26");
        putDefaultStringExtra(intent, "rustyxr.projectionBorderOpacity", "0.0");
        putDefaultStringExtra(intent, "rustyxr.projectionBorderPolicy", "passthrough-underlay");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchMode", "edge-stretch");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchCoreScale", "1.0");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchEdgeInsetUv", "0.008");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchMaxInsetUv", "0.070");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchCurve", "1.35");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchInnerBlendUv", "0.060");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchBlendCurve", "1.35");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchBlendMode", "target-inner-band");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchCornerMode", "target-footprint");
        putDefaultStringExtra(intent, "rustyxr.peripheralStretchDebug", "off");
        putDefaultStringExtra(intent, "rustyxr.mediaProjection", "false");
    }

    private static void putDefaultStringExtra(Intent intent, String key, String value) {
        if (!intent.hasExtra(key)) {
            intent.putExtra(key, value);
        }
    }

    private void requestMediaProjectionIfEnabled() {
        if (!booleanExtra("rustyxr.mediaProjection", false)) {
            return;
        }
        long delayMs = longExtra("rustyxr.mediaProjectionDelayMs", DEFAULT_MEDIA_PROJECTION_DELAY_MS);
        mainHandler.postDelayed(new Runnable() {
            @Override
            public void run() {
                requestMediaProjection();
            }
        }, Math.max(0L, delayMs));
    }

    private void requestMediaProjection() {
        if (mediaProjectionManager == null) {
            Log.w(TAG, "MediaProjectionManager is unavailable");
            return;
        }
        Log.i(TAG, "Requesting MediaProjection consent");
        startActivityForResult(
            mediaProjectionManager.createScreenCaptureIntent(),
            MEDIA_PROJECTION_REQUEST);
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != MEDIA_PROJECTION_REQUEST) {
            return;
        }
        if (resultCode != RESULT_OK || data == null) {
            Log.w(TAG, "MediaProjection consent denied or cancelled");
            return;
        }
        Log.i(TAG, "MediaProjection consent granted; starting stream service");
        Intent serviceIntent = new Intent(this, MediaProjectionStreamService.class);
        serviceIntent.putExtra(MediaProjectionStreamService.EXTRA_RESULT_CODE, resultCode);
        serviceIntent.putExtra(MediaProjectionStreamService.EXTRA_RESULT_DATA, data);
        serviceIntent.putExtra(MediaProjectionStreamService.EXTRA_HOST, "127.0.0.1");
        serviceIntent.putExtra(MediaProjectionStreamService.EXTRA_PORT, intExtra("rustyxr.mediaProjectionPort", 8787));
        serviceIntent.putExtra(MediaProjectionStreamService.EXTRA_WIDTH, intExtra("rustyxr.mediaProjectionWidth", 512));
        serviceIntent.putExtra(MediaProjectionStreamService.EXTRA_HEIGHT, intExtra("rustyxr.mediaProjectionHeight", 288));
        startForegroundService(serviceIntent);
    }

    @Override
    protected void onDestroy() {
        stopService(new Intent(this, MediaProjectionStreamService.class));
        super.onDestroy();
    }

    private boolean booleanExtra(String key, boolean fallback) {
        Intent intent = getIntent();
        if (intent == null || !intent.hasExtra(key)) {
            return fallback;
        }
        Object value = intent.getExtras().get(key);
        if (value instanceof Boolean) {
            return ((Boolean) value).booleanValue();
        }
        if (value instanceof String) {
            String text = ((String) value).trim().toLowerCase();
            return "true".equals(text) || "1".equals(text) || "yes".equals(text) || "on".equals(text);
        }
        return fallback;
    }

    private int intExtra(String key, int fallback) {
        Intent intent = getIntent();
        if (intent == null || !intent.hasExtra(key)) {
            return fallback;
        }
        try {
            Object value = intent.getExtras().get(key);
            if (value instanceof Number) {
                return ((Number) value).intValue();
            }
            if (value instanceof String) {
                return Integer.parseInt(((String) value).trim());
            }
        } catch (RuntimeException ignored) {
        }
        return fallback;
    }

    private long longExtra(String key, long fallback) {
        Intent intent = getIntent();
        if (intent == null || !intent.hasExtra(key)) {
            return fallback;
        }
        try {
            Object value = intent.getExtras().get(key);
            if (value instanceof Number) {
                return ((Number) value).longValue();
            }
            if (value instanceof String) {
                return Long.parseLong(((String) value).trim());
            }
        } catch (RuntimeException ignored) {
        }
        return fallback;
    }
}
