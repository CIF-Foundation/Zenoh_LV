
#include "zenoh.h"
#include <mutex>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <vector>

#if defined(_WIN32)
#define DLLEXPORT extern "C" __declspec(dllexport)
#elif defined(__GNUC__) || defined(__clang__)
// This covers NI Linux RT, Ubuntu, and macOS
#define DLLEXPORT extern "C" __attribute__((visibility("default")))
#else
#define DLLEXPORT extern "C"
#endif

// Helper to cast our pointer back to a Zenoh session struct
typedef struct {
  z_owned_session_t session;
} SessionHandle;

// 1. Open Session: Returns a pointer to a session handle
DLLEXPORT void *ZenohOpen() {
  SessionHandle *handle = (SessionHandle *)malloc(sizeof(SessionHandle));

  // In newer Zenoh-C, you allocate the struct first, then pass its pointer
  z_owned_config_t config;
  z_config_default(&config); // This initializes the struct in-place

  // Open the session
  if (z_open(&handle->session, z_move(config), NULL) < 0) {
    free(handle);
    return NULL;
  }
  return (void *)handle;
}

// 2. Put Data: LabVIEW passes the session pointer, key string, and value string
DLLEXPORT int ZenohPut(void *sessionPtr, const char *key, const char *value) {
  if (!sessionPtr)
    return -1;
  SessionHandle *handle = (SessionHandle *)sessionPtr;

  z_view_keyexpr_t key_expr;
  z_view_keyexpr_from_str(&key_expr, key);

  z_owned_bytes_t payload;
  z_bytes_from_static_str(&payload, value);

  // Perform the put operation
  return z_put(z_loan(handle->session), z_loan(key_expr), z_move(payload),
               NULL);
}

// 3. Close Session: Frees the Zenoh session and the allocated handle memory
DLLEXPORT void ZenohClose(void *sessionPtr) {
  if (!sessionPtr)
    return;
  SessionHandle *handle = (SessionHandle *)sessionPtr;

  z_drop(z_move(handle->session));
  free(handle);
}

// This accepts any raw buffer from LabVIEW
DLLEXPORT int ZenohPutRaw(void *sessionPtr, const char *key,
                          const uint8_t *data, size_t dataLen) {
  if (!sessionPtr)
    return -1;
  SessionHandle *handle = (SessionHandle *)sessionPtr;

  z_view_keyexpr_t key_expr;
  z_view_keyexpr_from_str(&key_expr, key);

  // Use z_bytes_copy_from_buf to handle raw binary data of any length
  z_owned_bytes_t payload;
  z_bytes_copy_from_buf(&payload, data, dataLen);

  return z_put(z_loan(handle->session), z_loan(key_expr), z_move(payload),
               NULL);
}

// This version allows you to set a MIME-type encoding for the payload
DLLEXPORT int ZenohPutWithEncoding(void *sessionPtr, const char *key,
                                   const uint8_t *data, size_t dataLen,
                                   const char *mime_type) {
  if (!sessionPtr)
    return -1;
  SessionHandle *handle = (SessionHandle *)sessionPtr;

  z_view_keyexpr_t key_expr;
  z_view_keyexpr_from_str(&key_expr, key);

  z_owned_bytes_t payload;
  z_bytes_copy_from_buf(&payload, data, dataLen);

  // Set up Put Options with Encoding
  z_put_options_t opts;
  z_put_options_default(&opts);

  // Convert string mime_type to Zenoh encoding
  z_owned_encoding_t encoding;
  z_encoding_from_str(&encoding, mime_type);
  opts.encoding = z_move(encoding);

  return z_put(z_loan(handle->session), z_loan(key_expr), z_move(payload),
               &opts);
}

// Helper: Append trace to "zenoh_debug.log"
void Log(const char *fmt, ...) {
  FILE *f = fopen("d:\\dev\\zenoh\\wrapper\\zenoh_debug.log", "a");
  if (f) {
    va_list args;
    va_start(args, fmt);
    vfprintf(f, fmt, args);
    va_end(args);
    // Ensure newlines
    // fprintf(f, "\n");
    fclose(f);
  }
}

// Helper to cast our pointer back to a Zenoh session struct
// typedef struct {
//   z_owned_session_t session;
// } SessionHandle;

// Structure to hold state for a single reader/subscriber channel
struct ReaderContext {
  std::vector<uint8_t> payload;
  std::mutex mtx;
  bool new_data = false;
  z_owned_subscriber_t sub;
};

// Callback now uses context to find the specific ReaderContext
void zenoh_sub_callback(z_loaned_sample_t *sample, void *context) {
  if (!context) {
    Log("Callback: context is NULL!\n");
    return;
  }
  ReaderContext *ctx = (ReaderContext *)context;

  std::lock_guard<std::mutex> lock(ctx->mtx);

  // 1. Get the opaque bytes from the sample
  const z_loaned_bytes_t *bytes = z_sample_payload(sample);

  // 2. Convert the bytes to a slice to get the raw pointer
  // We use an "owned" slice to hold the reference temporarily
  z_owned_slice_t owned_slice;
  z_bytes_to_slice(bytes, &owned_slice);

  // 3. Extract the pointer and length using slice accessors
  const uint8_t *start = z_slice_data(z_loan(owned_slice));
  size_t len = z_slice_len(z_loan(owned_slice));

  if (len > 0 && start != nullptr) {
    ctx->payload.assign(start, start + len);
    ctx->new_data = true;
  } else {
    Log("Callback: payload empty or null\n");
  }

  // 4. Must drop the owned slice
  z_drop(z_move(owned_slice));
}

// --- DLL Exports for LabVIEW ---

// 1. Declare the subscriber - Returns a ReaderContext handle (pointer)
DLLEXPORT void *ZenohReadStart(void *sessionPtr, const char *key) {
  z_owned_session_t *s = (z_owned_session_t *)sessionPtr;
  z_view_keyexpr_t ke;
  z_view_keyexpr_from_str(&ke, key);
  // Allocate new context for this reader
  ReaderContext *ctx = new ReaderContext();
  // Initialize subscriber struct to 0
  memset(&ctx->sub, 0, sizeof(z_owned_subscriber_t));
  // Setup the callback closure
  // Pass ctx as the context to the callback
  z_owned_closure_sample_t cb;
  z_closure_sample(&cb, zenoh_sub_callback, NULL, ctx);
  // Declare subscriber
  if (z_declare_subscriber(z_loan(*s), &ctx->sub, z_loan(ke), z_move(cb),
                           NULL) < 0) {
    delete ctx;
    return NULL;
  }
  return (void *)ctx;
}

// 2. Poll for data (The "Read" operation)
// Returns 1 if new data was found, 0 if not.
// Now takes a readerHandle instead of using global state
DLLEXPORT int ZenohReadPoll(void *readerHandle, uint8_t *buffer,
                            size_t bufferMax, size_t *actualLen) {
  if (!readerHandle)
    return 0;
  ReaderContext *ctx = (ReaderContext *)readerHandle;
  std::lock_guard<std::mutex> lock(ctx->mtx);
  if (!ctx->new_data) {
    return 0;
  }
  size_t len = ctx->payload.size();
  if (len > bufferMax) {
    len = bufferMax; // Safety clip
  }
  if (buffer) {
    memcpy(buffer, ctx->payload.data(), len);
  }
  if (actualLen) {
    *actualLen = len;
  }
  ctx->new_data = false; // Mark as read
  return 1;
}

// 3. Cleanup
DLLEXPORT void ZenohReadStop(void *readerHandle) {
  if (!readerHandle)
    return;
  ReaderContext *ctx = (ReaderContext *)readerHandle;
  if (z_internal_check(ctx->sub)) {
    z_undeclare_subscriber(z_move(ctx->sub));
  }
  delete ctx;
}
