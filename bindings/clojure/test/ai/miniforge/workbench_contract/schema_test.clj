;; Title: Miniforge App Foundation
;; Subtitle: Clojure binding tests — the golden fixtures validate against the mirror
;; Author: Christopher Lester
;; Copyright 2025-2026 Christopher Lester (christopher@miniforge.ai)
;;
;; Licensed under the Apache License, Version 2.0 (the "License");
;; you may not use this file except in compliance with the License.
;; You may obtain a copy of the License at
;;
;;     http://www.apache.org/licenses/LICENSE-2.0
;;
;; Unless required by applicable law or agreed to in writing, software
;; distributed under the License is distributed on an "AS IS" BASIS,
;; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
;; See the License for the specific language governing permissions and
;; limitations under the License.

(ns ai.miniforge.workbench-contract.schema-test
  "The golden fixtures are the cross-language contract test. Every
   registry and snapshot under fixtures/<tenant>/ round-trips through the
   Rust types, so each must also validate against this Malli mirror."
  (:require
   [ai.miniforge.workbench-contract.schema :as sut]
   [cheshire.core :as json]
   [clojure.java.io :as io]
   [clojure.test :refer [deftest is testing]]
   [malli.core :as m]
   [malli.error :as me]))

(def fixtures-dir
  "The repository's fixtures, relative to bindings/clojure (the test task's
   working directory)."
  (io/file "../../fixtures"))

(defn- tenant-dirs
  []
  (->> (.listFiles fixtures-dir)
       (filter #(.isDirectory %))
       (sort-by #(.getName %))))

(defn- read-json
  [file]
  (json/parse-string (slurp file) true))

(defn- explain
  [schema value]
  (pr-str (me/humanize (m/explain schema value))))

(deftest every-golden-fixture-validates-test
  (is (seq (tenant-dirs)) "no fixtures found; run from bindings/clojure")
  (doseq [dir (tenant-dirs)]
    (testing (.getName dir)
      (let [registry (read-json (io/file dir "registry.json"))
            snapshot (read-json (io/file dir "snapshot.json"))]
        (is (m/validate sut/StateVarRegistry registry)
            (explain sut/StateVarRegistry registry))
        (is (m/validate sut/WorkbenchSnapshot snapshot)
            (explain sut/WorkbenchSnapshot snapshot))))))

(deftest mirror-rejects-off-contract-snapshots-test
  (let [snapshot (read-json (io/file fixtures-dir "miniforge" "snapshot.json"))]
    (testing "a status outside the StateStatus enum"
      (is (not (m/validate sut/WorkbenchSnapshot
                           (assoc-in snapshot [:evaluations 0 :status] "green")))))
    (testing "a missing signature"
      (is (not (m/validate sut/WorkbenchSnapshot (dissoc snapshot :signature)))))))
