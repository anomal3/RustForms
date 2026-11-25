using System;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Threading;
using System.Windows.Threading;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace RustForms.VisualStudio.Designer
{
    /// <summary>Процесс «rustforms-designer serve»: команда строкой JSON, ответ строкой JSON.</summary>
    public sealed class Session : IDisposable
    {
        private readonly Process process;
        private readonly BlockingCollection<Tuple<string, Action<JObject>>> queue =
            new BlockingCollection<Tuple<string, Action<JObject>>>();
        private readonly Dispatcher dispatcher = Dispatcher.CurrentDispatcher;

        public Session(string designer, string form)
        {
            process = Process.Start(new ProcessStartInfo(designer, "serve \"" + form + "\"")
            {
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardInput = true,
                RedirectStandardOutput = true,
                StandardOutputEncoding = Encoding.UTF8,
            });
            var worker = new Thread(Work) { IsBackground = true, Name = "rustforms-session" };
            worker.Start();
        }

        /// <summary>Ответы приходят в поток интерфейса в том же порядке, что и команды.</summary>
        public void Send(JObject request, Action<JObject> reply)
        {
            if (!queue.IsAddingCompleted)
            {
                queue.Add(Tuple.Create(request.ToString(Formatting.None), reply));
            }
        }

        private void Work()
        {
            var input = new StreamWriter(process.StandardInput.BaseStream, new UTF8Encoding(false));
            foreach (var item in queue.GetConsumingEnumerable())
            {
                JObject answer;
                try
                {
                    input.Write(item.Item1);
                    input.Write('\n');
                    input.Flush();
                    var line = process.StandardOutput.ReadLine();
                    if (line == null)
                    {
                        throw new IOException("дизайнер завершился");
                    }
                    answer = JObject.Parse(line);
                }
                catch (Exception e) when (e is IOException || e is JsonException)
                {
                    answer = new JObject { ["error"] = "Нет связи с дизайнером: " + e.Message };
                }
                var then = item.Item2;
                dispatcher.BeginInvoke(new Action(() => then(answer)));
            }
            // Дизайнер выходит сам, когда закрывается его ввод.
            try
            {
                process.StandardInput.Close();
            }
            catch (IOException)
            {
            }
        }

        public void Dispose() => queue.CompleteAdding();
    }
}
