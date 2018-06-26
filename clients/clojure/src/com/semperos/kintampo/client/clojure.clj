(ns com.semperos.kintampo.client.clojure
  (:import [org.zeromq ZMQ ZContext])
  (:gen-class))

(defn setup-subscriber [sub dir]
  (.connect sub "tcp://localhost:5563")
  (.subscribe sub (.getBytes dir ZMQ/CHARSET)))

(defn subscribe [kintampo-dir]
  (let [ctx (ZContext.)
        sub (.createSocket ctx ZMQ/SUB)]
    (println "Subscribing on tcp://localhost:5563 envelope" (pr-str kintampo-dir))
    (setup-subscriber sub kintampo-dir)
    (.disconnect sub "tcp://localhost:5563")
    (setup-subscriber sub kintampo-dir)
    (println "Clojure subscriber ready for messages.")
    (while (not (.isInterrupted (Thread/currentThread)))
      (let [envelope (.recvStr sub)
            _ (println "Got envelope:" (pr-str envelope))
            message (.recvStr sub)
            _ (println "Got message:" (pr-str message))]
        (println "Clojure received:" (pr-str envelope) " with message " (pr-str message))))))

(defn -main
  [& args]
  (println "Setting up Clojure subscriber...")
  (subscribe (first args)))
