using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

namespace ConsoleApp3
{
    class Program
    {
        const string dllPath = @"C:\DEV\Escapi\escapi\target\release\escapi_rust.dll";
        //const string dllPath = @"C:\DEV\Escapi\escapi\target\debug\escapi_rust.dll";


        [STAThread]
        static int Main(string[] args)
        {
            Int64 device_ptr;
            uint camera_index = 0;
            uint width = 640;
            uint height = 480;
            float framerate = 30.0f;

            Int64 buffer;

            allocate_buffer_i32(width, height, out buffer);

            Int64 postProcessBuffer;
            allocate_buffer_u8(width * height * 4, out postProcessBuffer);

            UInt32 format;

            var result = init(camera_index, width, height, framerate, buffer, out device_ptr, out format);
            Console.WriteLine("format is " + format);
            if (result != 0)
            {
                Int64 str_ptr;
                result = get_device_name(device_ptr, out str_ptr);

                if (result != 0)
                {
                    byte[] bytes = new byte[result];
                    Marshal.Copy(new IntPtr(str_ptr), bytes, 0, result);
                    var deviceName = Encoding.UTF8.GetString(bytes);
                    Console.WriteLine("device name is " + deviceName);
                    free_string(str_ptr);
                }

                System.Diagnostics.Stopwatch stopwatch = new System.Diagnostics.Stopwatch();
                stopwatch.Start();

                var nbImageRead = 0;

                while (true)
                {
                    result = get_capture_buffer(device_ptr);
                    if (result != 0)
                    {
                        nbImageRead++;

                        stopwatch.Stop();
                        var elapsedMs = stopwatch.ElapsedMilliseconds;
                        stopwatch.Reset();
                        stopwatch.Start();

                        Console.WriteLine("got a new image in " + elapsedMs + " ms");

                        post_convert(postProcessBuffer, buffer, width, height, format);

                        if (nbImageRead == 10)
                        {
                            byte[] imageBytes = new byte[width * height * 4];
                            Marshal.Copy(new IntPtr(postProcessBuffer), imageBytes, 0, (int)(width * height * 4));
                            SavePPM(width, height, imageBytes);
                            Console.WriteLine("image saved");
                        }
                        //free_buffer(bufferPtr);
                    }
                }

                free_device(device_ptr);
            }
            else
            {
                Console.WriteLine("Cannot init camera");
            }
            Console.WriteLine("Program ended. Press a key to exit.");
            Console.ReadLine();
            return 0;
        }

        private static void SavePPM(uint width, uint height, byte[] imageBytes)
        {
            var imageBytesLength = imageBytes.Length;
            using (var writer = new StreamWriter("image.ppm"))
            {
                writer.WriteLine("P3");
                writer.WriteLine("" + width + " " + height);
                writer.WriteLine("255");
                for (int j = 0; j < height; j++)
                {
                    for (int i = 0; i < width ; i++)
                    {
                        for (int channel = 0; channel < 4; channel++)
                        {
                            if (channel < 3)
                            {
                                var baseIndex = j * width * 4 + i * 4;
                                var swapedChannelIndx = SwapChannelIndex(channel);
                                var index = baseIndex + swapedChannelIndx;
                                writer.Write("" + imageBytes[index] + " ");
                            }
                        }
                    }
                    writer.WriteLine("");
                }
            }
        }

        private static int SwapChannelIndex(int channel)
        {
            if (channel == 0)
            {
                return 2;
            }
            if (channel == 2)
            {
                return 0;
            }
            return channel;
        }

        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern Int32 init
        (
            UInt32 camIndex,
            UInt32 width,
            UInt32 height,
            float desired_fps,
            Int64 buffer,
            out Int64 intPtr,
            out UInt32 formatIndex
        );

        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern Int32 num_devices();
        
        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern Int32 get_device_name(Int64 device_ptr, out Int64 str_ptr_out);
        
        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern void free_string(Int64 str_ptr);

        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern void free_device(Int64 str_ptr);
        
        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern void allocate_buffer_i32(UInt32 width, UInt32 height, out Int64 buffer_ptr_out);
        
        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern void allocate_buffer_u8(UInt32 size, out Int64 buffer_ptr_out);

        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern Int32 get_capture_buffer(Int64 device_ptr);

        [DllImport(dllPath, CharSet = CharSet.Auto)]
        public static extern void post_convert(Int64 bufferOut, Int64 bufferIn, UInt32 width, UInt32 height, UInt32 format);

    }
}
